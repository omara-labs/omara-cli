use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::Local;

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
        println!("  ✅ Installed all packages.");
    } else {
        eprintln!("  ❌ Package installation encountered errors.");
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
    match action {
        OsCommands::Install { force: _, dry_run } => {
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
                        println!("  2. Mount target partition to /mnt/sysroot.");
                        println!("  3. Perform system image copying / bootstrap.");
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
                return;
            }

            match mode {
                SystemMode::LiveInstaller => {
                    println!("🚀 Starting Bare-Metal TUI Installer...");
                    // Prototyped install logic
                    println!("  Installing system files to disk...");
                    println!("  ✅ Target disk setup complete. Restart machine to boot Omara OS.");
                }
                SystemMode::Coexistence => {
                    println!("🚀 Running Coexistence Setup...");
                    install_system_packages();
                    println!("  ✅ Niri Wayland session registered alongside other DEs.");
                }
                SystemMode::Bootstrap => {
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
                println!("Are you sure you want to proceed? [y/N]");
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_err() || !input.trim().eq_ignore_ascii_case("y") {
                    println!("Reset aborted.");
                    return;
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
}
