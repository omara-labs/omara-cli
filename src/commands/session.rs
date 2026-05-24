use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum SessionCommands {
    /// Lock the Wayland compositor session
    Lock,

    /// Gracefully exit the Niri compositor session
    Logout,

    /// Put the system to sleep (suspend)
    Suspend,

    /// Reboot the system
    Reboot,

    /// Shut down and power off the system
    Poweroff,
}

fn get_omara_session() -> String {
    if let Ok(val) = std::env::var("OMARA_SESSION") {
        return val;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
    let conf_path = std::path::Path::new(&home).join(".config").join("omara").join("omara.conf");
    if conf_path.exists() {
        if let Ok(content) = std::fs::read_to_string(conf_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("OMARA_SESSION=") {
                    return trimmed["OMARA_SESSION=".len()..]
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }
    }
    "gnome".to_string()
}

pub fn run(action: &SessionCommands) {
    match action {
        SessionCommands::Lock => {
            println!("🔒 Locking session...");
            // Attempt standard systemd/loginctl lock
            let result = Command::new("loginctl")
                .arg("lock-session")
                .status();
            
            if !result.map(|s| s.success()).unwrap_or(false) {
                // Fallback: try swaylock
                let swaylock_result = Command::new("swaylock")
                    .args(["-f", "-c", "000000"])
                    .status();
                if !swaylock_result.map(|s| s.success()).unwrap_or(false) {
                    eprintln!("❌ Failed to lock session. Ensure a locker is active.");
                }
            }
        }
        SessionCommands::Logout => {
            let session = get_omara_session();
            if session == "gnome" {
                println!("🚪 Logging out of GNOME session...");
                let _ = Command::new("gnome-session-quit")
                    .args(["--logout", "--no-prompt"])
                    .status();
            } else {
                println!("🚪 Logging out of Niri session...");
                let status = Command::new("niri")
                    .args(["--msg", "action", "quit", "--no-confirm"])
                    .status();
                
                if !status.map(|s| s.success()).unwrap_or(false) {
                    // Fallback to loginctl terminate
                    let _ = Command::new("loginctl")
                        .args(["terminate-user", ""])
                        .status();
                }
            }
        }
        SessionCommands::Suspend => {
            println!("🌙 Suspending system...");
            let _ = Command::new("systemctl")
                .arg("suspend")
                .status();
        }
        SessionCommands::Reboot => {
            println!("🔄 Rebooting system...");
            let _ = Command::new("systemctl")
                .arg("reboot")
                .status();
        }
        SessionCommands::Poweroff => {
            println!("🔌 Powering off system...");
            let _ = Command::new("systemctl")
                .arg("poweroff")
                .status();
        }
    }
}
