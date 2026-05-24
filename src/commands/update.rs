use colored::Colorize;
use std::process::Command;

/// Run a command and return success status with output
fn run_update_command(cmd: &str, args: &[&str], label: &str) -> (bool, String) {
    println!("   {} {}", "→".yellow(), label);
    
    let output = Command::new(cmd)
        .args(args)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        if !line.trim().is_empty() {
                            println!("      {}", line);
                        }
                    }
                }
                (true, String::new())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                (false, stderr.into_owned())
            }
        }
        Err(e) => (false, format!("Failed to run: {}", e)),
    }
}

pub fn run() {
    println!("{}", "🔄  Omara System Update".bold().cyan());
    println!("   Keeping your machine fresh and secure.\n");

    let mut all_ok = true;

    // DNF update
    let (dnf_ok, dnf_err) = run_update_command("sudo", &["dnf", "upgrade", "--refresh", "-y"], "DNF packages");
    if !dnf_ok {
        all_ok = false;
        println!("      {}", format!("⚠️  DNF: {}", dnf_err).yellow());
    }

    println!();

    // Flatpak update
    let (flatpak_ok, flatpak_err) = run_update_command("flatpak", &["update", "-y"], "Flatpaks");
    if !flatpak_ok {
        all_ok = false;
        println!("      {}", format!("⚠️  Flatpak: {}", flatpak_err).yellow());
    }

    println!();

    if all_ok {
        println!("{}", "✅  Update complete. Enjoy your fresh Omara.".green().bold());
    } else {
        println!("{}", "⚠️  Update finished with some errors. Check output above.".yellow());
    }
}
