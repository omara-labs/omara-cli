use clap::Subcommand;
use colored::Colorize;
use std::process::Command;
use std::path::Path;

/// Check if a command exists in PATH
fn check_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a file/directory exists
fn check_path(path: &str) -> bool {
    let expanded = shellexpand::tilde(path).to_string();
    Path::new(&expanded).exists()
}

/// Check if a systemd service is active
fn check_service(service: &str) -> bool {
    Command::new("systemctl")
        .arg("is-active")
        .arg(service)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a package is installed via dnf
fn check_dnf_package(pkg: &str) -> bool {
    Command::new("dnf")
        .arg("list")
        .arg("installed")
        .arg(pkg)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a flatpak is installed
fn check_flatpak(pkg: &str) -> bool {
    Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg(pkg)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[derive(Subcommand)]
pub enum DoctorCommands {
    /// Run all health checks (default)
    Check,
    
    /// Quick basic checks
    Quick,
    
    /// Comprehensive system diagnostics
    Full,
}

/// Run quick health checks
fn run_quick(fix: bool) {
    println!("{}", "🩺  Omara Doctor - Quick Check".bold().cyan());
    println!();

    let mut all_good = true;
    let mut fix_niri_config = false;

    // Core system
    if check_command("rustc") {
        println!("  {} Rust toolchain", "✓".green());
    } else {
        println!("  {} Rust toolchain", "✗".red());
        all_good = false;
    }

    if check_command("omara") {
        println!("  {} omara CLI", "✓".green());
    } else {
        println!("  {} omara CLI", "✗".red());
        all_good = false;
    }

    // Niri config
    if check_path("~/.config/omara/niri/config.kdl") {
        println!("  {} Niri config", "✓".green());
    } else {
        println!("  {} Niri config", "✗".red());
        all_good = false;
        fix_niri_config = true;
    }

    println!();
    if all_good {
        println!("{}", "✅ Quick check passed!");
    } else {
        println!("{}", "⚠️  Some issues found.".yellow());
        if fix {
            println!();
            println!("{}", "🩹 Attempting automated healing...".bold().cyan());
            if fix_niri_config {
                println!("  → Restoring default Niri config...");
                crate::commands::app::reset_app("niri");
            }
            println!();
            println!("{}", "✅ Healing pass complete. Please re-run doctor to verify.".bold().green());
        }
    }
}

/// Run full comprehensive diagnostics
fn run_full(fix: bool) {
    println!("{}", "🩺  Omara Doctor - Full Diagnostics".bold().cyan());
    println!();

    let mut all_good = true;
    let mut issues: Vec<String> = Vec::new();
    let mut fixable_packages: Vec<String> = Vec::new();
    let mut fix_ollama_service = false;
    let mut fix_niri_config = false;
    let mut fix_gh_config = false;

    // Rust toolchain
    if check_command("rustc") {
        println!("  {} Rust toolchain", "✓".green());
    } else {
        println!("  {} Rust toolchain", "✗".red());
        issues.push("Rust not installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string());
        all_good = false;
    }

    // omara CLI
    if check_command("omara") {
        println!("  {} omara CLI", "✓".green());
    } else {
        println!("  {} omara CLI", "✗".red());
        issues.push("omara CLI not in PATH. Build and install from omara-cli".to_string());
        all_good = false;
    }

    // Ollama (optional)
    if check_command("ollama") {
        if check_service("ollama") {
            println!("  {} Ollama service", "✓".green());
        } else {
            println!("  {} Ollama service", "⚠️".yellow());
            issues.push("Ollama not running: sudo systemctl start ollama".to_string());
            fix_ollama_service = true;
        }
        
        let output = Command::new("ollama").arg("list").output();
        if let Ok(output) = output {
            let list = String::from_utf8_lossy(&output.stdout);
            if !list.trim().is_empty() && list.lines().count() > 1 {
                println!("  {} Ollama models", "✓".green());
            } else {
                println!("  {} Ollama models", "⚠️".yellow());
                issues.push("No Ollama models: ollama pull gemma2:2b".to_string());
            }
        }
    } else {
        println!("  {} Ollama", "○".bright_black());
    }

    // Niri config
    if check_path("~/.config/omara/niri/config.kdl") {
        println!("  {} Niri config (Omara)", "✓".green());
    } else if check_path("~/.config/niri/config.kdl") {
        println!("  {} Niri config (legacy)", "⚠️".yellow());
        issues.push("Migrate config to ~/.config/omara/niri/".to_string());
        fix_niri_config = true;
    } else {
        println!("  {} Niri config", "✗".red());
        issues.push("Manually link niri configs from omara-configs".to_string());
        all_good = false;
        fix_niri_config = true;
    }

    // gh config
    if check_path("~/.config/gh/config.yml") && check_path("~/.config/gh/hosts.yml") {
        println!("  {} gh config", "✓".green());
    } else {
        println!("  {} gh config", "✗".red());
        issues.push("Restore gh config: omara app reset gh".to_string());
        all_good = false;
        fix_gh_config = true;
    }

    // Required packages
    let required_pkgs = [
        ("niri", false),
        ("quickshell", false),
        ("swaync", false),
        ("kitty", false),
        ("fish", false),
        ("fastfetch", false),
        ("wl-clipboard", false),
        ("gh", false),
    ];
    
    for (pkg, optional) in required_pkgs {
        if check_dnf_package(pkg) || check_flatpak(pkg) {
            println!("  {} {}", "✓".green(), pkg);
        } else {
            if optional {
                println!("  {} {} (optional)", "○".bright_black(), pkg);
            } else {
                println!("  {} {}", "✗".red(), pkg);
                issues.push(format!("Install: sudo dnf install {}", pkg));
                all_good = false;
                fixable_packages.push(pkg.to_string());
            }
        }
    }

    println!();
    if all_good {
        println!("{}", "✅ All checks passed. Your Omara system is healthy!".bold().green());
    } else {
        println!("{}", "⚠️  Issues found:".yellow());
        for issue in &issues {
            println!("   - {}", issue);
        }

        if fix {
            println!();
            println!("{}", "🩹 Attempting automated healing...".bold().cyan());

            if fix_ollama_service {
                println!("  → Starting Ollama service...");
                let _ = Command::new("sudo")
                    .args(["systemctl", "start", "ollama"])
                    .status();
            }

            if fix_niri_config {
                println!("  → Restoring default Niri config...");
                crate::commands::app::reset_app("niri");
            }

            if fix_gh_config {
                println!("  → Restoring default gh config...");
                crate::commands::app::reset_app("gh");
            }

            if !fixable_packages.is_empty() {
                println!("  → Installing missing packages: {}...", fixable_packages.join(", "));
                let mut cmd = Command::new("sudo");
                cmd.arg("dnf").arg("install").arg("-y");
                for pkg in &fixable_packages {
                    cmd.arg(pkg);
                }
                let status = cmd.status();
                if status.map(|s| s.success()).unwrap_or(false) {
                    println!("    ✅ Packages installed successfully.");
                } else {
                    eprintln!("    ❌ Package installation failed.");
                }
            }

            println!();
            println!("{}", "✅ Healing pass complete. Please re-run doctor to verify.".bold().green());
        }
    }
}

/// Run all checks (same as check subcommand)
fn run_check(fix: bool) {
    run_full(fix);
}

pub fn run(command: &DoctorCommands, fix: bool) {
    match command {
        DoctorCommands::Check => run_check(fix),
        DoctorCommands::Quick => run_quick(fix),
        DoctorCommands::Full => run_full(fix),
    }
}

/// Default action: run full check
pub fn run_default(fix: bool) {
    run_full(fix);
}
