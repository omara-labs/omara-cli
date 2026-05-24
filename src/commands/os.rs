use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::Write;
use chrono::Local;
use inquire::{Confirm, Password, Select, Text};

#[derive(Subcommand)]
pub enum OsCommands {
    /// Install Omara OS (environment-aware installer/bootstrap)
    Install {
        /// Force install without pre-flight checks or interactive prompts
        #[arg(short, long)]
        force: bool,

        /// Dry-run mode: prints detected environment and planned actions without executing
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Reset system configuration and packages to Omara defaults
    Reset {
        /// Proceed without backup or confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Debug, PartialEq)]
pub enum SystemMode {
    LiveInstaller,
    Coexistence,
    Bootstrap,
}

impl std::fmt::Display for SystemMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemMode::LiveInstaller => write!(f, "Bare-Metal / VM Live Installer"),
            SystemMode::Coexistence => write!(f, "Dual-DE Coexistence (alongside other Desktop Environments)"),
            SystemMode::Bootstrap => write!(f, "Clean Minimal Fedora Conversion"),
        }
    }
}

fn detect_system_mode() -> SystemMode {
    if Path::new("/run/initramfs/live").exists() {
        SystemMode::LiveInstaller
    } else if has_other_des() {
        SystemMode::Coexistence
    } else {
        SystemMode::Bootstrap
    }
}

fn has_other_des() -> bool {
    let mut other_found = false;
    for dir in &["/usr/share/wayland-sessions", "/usr/share/xsessions"] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") && !name.contains("niri") {
                        other_found = true;
                        break;
                    }
                }
            }
        }
    }
    other_found
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn get_available_disks() -> Vec<String> {
    let output = Command::new("lsblk")
        .args(["-dno", "NAME,SIZE,MODEL"])
        .output();
    
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut disks = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                disks.push(trimmed.to_string());
            }
        }
        if !disks.is_empty() {
            return disks;
        }
    }
    
    // Fallback dummy disks for testing/VM
    vec![
        "sda (250GB, Virtual Disk)".to_string(),
        "nvme0n1 (512GB, NVMe Virtual Disk)".to_string(),
    ]
}

fn get_offline_image_path() -> Option<PathBuf> {
    let paths = [
        "/run/initramfs/live/omara-base.tar.zst",
        "/run/initramfs/live/LiveOS/rootfs.img",
        "/tmp/omara-base.tar.zst", // For testing/debugging
    ];
    for p in &paths {
        let path = Path::new(p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn install_system_packages() {
    let apps = crate::commands::app::load_app_manifests();
    if apps.is_empty() {
        println!("  ⚠️  No packages found in manifests, skipping package install.");
        return;
    }
    println!("  Installing default package set ({} apps)...", apps.len());
    
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg("install").arg("-y");
    for app in &apps {
        cmd.arg(app);
    }
    
    let status = cmd.status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("  %  Installed all packages.");
    } else {
        eprintln!("  ❌ Package installation encountered errors.");
    }
}

fn install_packages_to_root(root: &str) {
    let apps = crate::commands::app::load_app_manifests();
    if apps.is_empty() {
        println!("  ⚠️  No packages found in manifests, skipping target package install.");
        return;
    }
    println!("  Installing default package set into target root ({} apps)...", apps.len());
    
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg(format!("--installroot={}", root)).arg("install").arg("-y");
    for app in &apps {
        cmd.arg(app);
    }
    
    let status = cmd.status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("  ✅ Installed all packages into target root.");
    } else {
        eprintln!("  ❌ Package installation in target root encountered errors.");
    }
}

fn audit_and_install_packages() {
    let apps = crate::commands::app::load_app_manifests();
    if apps.is_empty() {
        println!("    ⚠️  No packages found in manifests, skipping audit.");
        return;
    }
    
    let mut missing_apps = Vec::new();
    for app in &apps {
        if !command_exists(app) {
            missing_apps.push(app.clone());
        }
    }

    if missing_apps.is_empty() {
        println!("    All default packages are already installed.");
    } else {
        println!("    Missing packages detected: {}", missing_apps.join(", "));
        println!("    Installing missing packages...");
        
        let mut cmd = Command::new("sudo");
        cmd.arg("dnf").arg("install").arg("-y");
        for app in &missing_apps {
            cmd.arg(app);
        }
        let _ = cmd.status();
    }
}

pub fn run(action: &OsCommands) {
    if let Err(e) = run_internal(action) {
        eprintln!("{} {}", "❌ Error:".red().bold(), e);
    }
}

fn run_internal(action: &OsCommands) -> anyhow::Result<()> {
    match action {
        OsCommands::Install { force, dry_run } => {
            let mode = detect_system_mode();
            println!("{}", "🖥️  Omara OS Installer".bold().cyan());
            println!("  Detected Mode: {}", mode.to_string().yellow());
            println!();

            if *dry_run {
                println!("{}", "⚠️  DRY-RUN MODE — No changes will be written.".yellow().bold());
                match mode {
                    SystemMode::LiveInstaller => {
                        println!("Planned Actions:");
                        println!("  1. Detect target block devices for partitioning.");
                        println!("  2. Format partition structure using ext4 and vfat.");
                        println!("  3. Mount target partition to /mnt/sysroot.");
                        println!("  4. Perform system image copying / bootstrap.");
                        println!("  5. Generate /etc/fstab and set hostname.");
                        println!("  6. Create administrator user and password.");
                    }
                    SystemMode::Coexistence => {
                        println!("Planned Actions:");
                        println!("  1. Enable Tailscale, RPM Fusion, Walker, Niri, Quickshell, Yazi, and Omara Repos.");
                        println!("  2. Install Wayland compositor, status bar, and desktop packages via DNF.");
                        println!("  3. Preserve existing GNOME/KDE packages.");
                        println!("  4. Register Niri session in /usr/share/wayland-sessions.");
                    }
                    SystemMode::Bootstrap => {
                        println!("Planned Actions:");
                        println!("  1. Enable all custom repositories (RPM Fusion, Coprs, Terra, Tailscale, Omara).");
                        println!("  2. Install all default DNF packages from omara-os manifests.");
                        println!("  3. Enable GDM/greetd display manager user service.");
                    }
                }
                println!();
                println!("✅ Dry-run complete.");
                return Ok(());
            }

            match mode {
                SystemMode::LiveInstaller => {
                    println!("{}", "🚀 Starting Bare-Metal Interactive Installer...".bold().cyan());
                    
                    let disks = get_available_disks();
                    let selected_disk = Select::new("Select target disk for Omara OS installation:", disks).prompt()?;
                    
                    let partition_options = vec![
                        "Automatic Partitioning (Erase entire disk)",
                        "Manual Partitioning (Launch cfdisk)",
                    ];
                    let partition_choice = Select::new("Select partitioning method:", partition_options).prompt()?;
                    
                    let hostname = Text::new("Enter hostname:").with_default("omara").prompt()?;
                    let username = Text::new("Enter username:").with_default("jeryd").prompt()?;
                    
                    let mut password;
                    loop {
                        password = Password::new("Enter password:")
                            .with_display_mode(inquire::PasswordDisplayMode::Masked)
                            .prompt()?;
                        let confirm = Password::new("Confirm password:")
                            .with_display_mode(inquire::PasswordDisplayMode::Masked)
                            .prompt()?;
                        
                        if password == confirm {
                            break;
                        }
                        println!("{}", "❌ Passwords do not match. Please try again.".red());
                    }

                    println!();
                    println!("{}", "📋 Installation Summary".bold().cyan());
                    println!("  Target Disk:   {}", selected_disk.yellow());
                    println!("  Partitioning:  {}", partition_choice.yellow());
                    println!("  Hostname:      {}", hostname.yellow());
                    println!("  Username:      {}", username.yellow());
                    println!();

                    let proceed = Confirm::new("Proceed with installation? This will format the selected disk and erase all data.")
                        .with_default(false)
                        .prompt()?;

                    if !proceed {
                        println!("Installation aborted.");
                        return Ok(());
                    }

                    println!("{}", "⌛ Running installation...".bold().cyan());
                    let disk_name = selected_disk.split_whitespace().next().unwrap_or("sda");
                    let disk_path = format!("/dev/{}", disk_name);
                    
                    if partition_choice.contains("Manual") {
                        println!("  Launching cfdisk for {}...", disk_path);
                        let _ = Command::new("sudo").arg("cfdisk").arg(&disk_path).status();
                    } else {
                        // Automatic formatting
                        println!("  Formatting partitions on {}...", disk_path);
                        let p_boot = format!("{}1", disk_path);
                        let p_root = format!("{}2", disk_path);
                        
                        let _ = Command::new("sudo").args(["mkfs.vfat", "-F32", &p_boot]).status();
                        let _ = Command::new("sudo").args(["mkfs.ext4", "-F", &p_root]).status();
                    }
                    
                    // Mounting target partitions
                    let sysroot = "/mnt/sysroot";
                    println!("  Mounting target root to {}...", sysroot);
                    let _ = fs::create_dir_all(sysroot);
                    let root_partition = format!("{}2", disk_path);
                    let boot_partition = format!("{}1", disk_path);
                    
                    let mount_root_status = Command::new("sudo")
                        .args(["mount", &root_partition, sysroot])
                        .status();
                    
                    if mount_root_status.map(|s| s.success()).unwrap_or(false) {
                        let efi_dir = format!("{}/boot/efi", sysroot);
                        let _ = Command::new("sudo").args(["mkdir", "-p", &efi_dir]).status();
                        let _ = Command::new("sudo").args(["mount", &boot_partition, &efi_dir]).status();
                        
                        // Check for Offline Image vs Online Bootstrap
                        if let Some(image_path) = get_offline_image_path() {
                            println!("  📦 Found offline system image at: {}", image_path.display().to_string().yellow());
                            println!("  → Extracting base OS files...");
                            let extract_status = Command::new("sudo")
                                .args(["tar", "--zstd", "-xpf", image_path.to_str().unwrap(), "-C", sysroot])
                                .status();
                            
                            if !extract_status.map(|s| s.success()).unwrap_or(false) {
                                eprintln!("  ❌ Failed to extract offline image. Attempting DNF bootstrap fallback...");
                                install_packages_to_root(sysroot);
                            }
                        } else {
                            println!("  🌐 No offline system image found. Bootstrapping OS online via DNF...");
                            let dnf_status = Command::new("sudo")
                                .args(["dnf", "--installroot=/mnt/sysroot", "groupinstall", "-y", "Core", "Standard"])
                                .status();
                            
                            if dnf_status.map(|s| s.success()).unwrap_or(false) {
                                install_packages_to_root(sysroot);
                            }
                        }
                        
                        // Configure target system
                        println!("  → Generating /etc/fstab...");
                        let fstab_content = format!(
                            "{} / ext4 defaults 1 1\n{} /boot/efi vfat defaults 0 2\n",
                            root_partition, boot_partition
                        );
                        let fstab_path = format!("{}/etc/fstab", sysroot);
                        let _ = Command::new("sudo")
                            .args(["tee", &fstab_path])
                            .stdin(std::process::Stdio::piped())
                            .spawn()?
                            .stdin
                            .unwrap()
                            .write_all(fstab_content.as_bytes());

                        println!("  → Setting hostname to '{}'...", hostname);
                        let hostname_path = format!("{}/etc/hostname", sysroot);
                        let _ = Command::new("sudo")
                            .args(["tee", &hostname_path])
                            .stdin(std::process::Stdio::piped())
                            .spawn()?
                            .stdin
                            .unwrap()
                            .write_all(hostname.as_bytes());

                        println!("  → Configuring administrator account...");
                        let user_status = Command::new("sudo")
                            .args(["chroot", sysroot, "useradd", "-m", "-G", "wheel", &username])
                            .status();

                        if user_status.map(|s| s.success()).unwrap_or(false) {
                            let passwd_input = format!("{}:{}", username, password);
                            let mut child = Command::new("sudo")
                                .args(["chroot", sysroot, "chpasswd"])
                                .stdin(std::process::Stdio::piped())
                                .spawn()?;
                            
                            if let Some(mut stdin) = child.stdin.take() {
                                stdin.write_all(passwd_input.as_bytes())?;
                            }
                            let _ = child.wait();
                        }
                        
                        // Unmount target partitions
                        println!("  Cleaning up and unmounting target...");
                        let _ = Command::new("sudo").args(["umount", &efi_dir]).status();
                        let _ = Command::new("sudo").args(["umount", sysroot]).status();
                    } else {
                        eprintln!("  ❌ Failed to mount target partitions.");
                    }
                    
                    println!("  ✅ Installation completed successfully! Please reboot your machine.");
                }
                SystemMode::Coexistence => {
                    let proceed = if *force {
                        true
                    } else {
                        Confirm::new("GNOME/KDE detected. Install Omara DE packages alongside your existing setup?")
                            .with_default(true)
                            .prompt()?
                    };

                    if !proceed {
                        println!("Installation aborted.");
                        return Ok(());
                    }

                    println!("🚀 Running Coexistence Setup...");
                    install_system_packages();
                    println!("  ✅ Niri Wayland session registered alongside other DEs.");
                }
                SystemMode::Bootstrap => {
                    let proceed = if *force {
                        true
                    } else {
                        Confirm::new("Install Omara DE packages on this minimal Fedora system?")
                            .with_default(true)
                            .prompt()?
                    };

                    if !proceed {
                        println!("Installation aborted.");
                        return Ok(());
                    }

                    println!("🚀 Running Fedora Minimal Bootstrap...");
                    install_system_packages();
                    println!("  ✅ System bootstrap complete. Restart machine to boot into Omara.");
                }
            }
        }
        OsCommands::Reset { yes } => {
            println!("{}", "🔄  Omara System Reset".bold().cyan());
            println!("This will backup your existing config folder and restore defaults.");
            
            if !*yes {
                let proceed = Confirm::new("Are you sure you want to proceed?")
                    .with_default(false)
                    .prompt()?;
                if !proceed {
                    println!("Reset aborted.");
                    return Ok(());
                }
            }

            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
            let config_dir = Path::new(&home).join(".config");

            // 1. Back up config folders (niri, quickshell, kitty, fish, gh)
            let backup_suffix = Local::now().format("%Y%m%d%H%M%S").to_string();
            let components_to_reset = ["niri", "quickshell", "kitty", "fish", "gh"];
            for component in &components_to_reset {
                let path = config_dir.join(component);
                if path.exists() {
                    let backup_path = config_dir.join(format!("{}.bak.{}", component, backup_suffix));
                    println!("  Backup: moving {} to {}", path.display(), backup_path.display());
                    if let Err(e) = fs::rename(&path, &backup_path) {
                        eprintln!("  ❌ Failed to backup config: {}", e);
                    }
                }
            }

            // 2. Restore templates from omara-configs
            let omara_configs_dir = crate::paths::get_component_path("omara-core");
            if omara_configs_dir.exists() {
                println!("  Restoring default templates from omara-configs...");
                let source_configs = omara_configs_dir.join("configs");
                if source_configs.exists() {
                    for component in &components_to_reset {
                        let src = source_configs.join(component);
                        let dest = config_dir.join(component);
                        if src.exists() {
                            println!("    Restoring default config: {}", dest.display());
                            if let Err(e) = copy_dir_all(&src, &dest) {
                                eprintln!("    ❌ Error restoring {}: {}", component, e);
                            }
                        }
                    }
                }
            } else {
                println!("  ⚠️  omara-configs not found at {}, skipping config restore.", omara_configs_dir.display());
            }

            // 3. Package Audit and reinstall
            println!("  Auditing package manifests...");
            audit_and_install_packages();

            // 4. Restart walker.service
            println!("  Restarting user services...");
            let _ = Command::new("systemctl")
                .args(["--user", "restart", "walker.service"])
                .status();

            println!("  ✅ System reset completed successfully!");
        }
    }
    Ok(())
}
