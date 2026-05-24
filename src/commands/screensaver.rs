use clap::Subcommand;
use colored::Colorize;
use std::process::Command;

/// Known screensaver binaries
const SCREENSAVER_BINS: &[&str] = &[
    "omara-screensaver-bounce",
    "omara-screensaver-matrix",
    "omara-screensaver-beams",
    "omara-screensaver-pour",
    "omara-screensaver-unstable",
];

#[derive(Subcommand)]
pub enum ScreensaverCommands {
    /// List available screensavers
    List,

    /// Launch a specific screensaver
    Start {
        /// Name of the screensaver to launch
        name: String,
    },

    /// Stop any running screensaver
    Stop,
}

/// Check if a screensaver binary exists
fn screensaver_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// List available screensavers
fn list_screensavers() {
    println!("{}", "🎨  Available Screensavers".bold().cyan());
    
    for bin in SCREENSAVER_BINS {
        if screensaver_exists(bin) {
            println!("  {}  {}", "●".green(), bin);
        } else {
            println!("  {}  {} (not installed)", "○".bright_black(), bin);
        }
    }
    
    println!("\n  Install: cargo install --path omara-art/screensavers");
}

/// Launch a specific screensaver
fn start_screensaver(name: &str) {
    let normalized = if name.ends_with("-screensaver") || name.ends_with("-bounce") {
        name.to_string()
    } else {
        format!("omara-screensaver-{}", name)
    };
    
    if screensaver_exists(&normalized) {
        println!("{} Launching {}...", "→".yellow(), normalized);
        let _ = Command::new(&normalized).status();
    } else {
        println!("{}", format!("❌ Screensaver '{}' not found", name).red());
        println!("  Available:");
        list_screensavers();
    }
}

/// Stop running screensaver
fn stop_screensaver() {
    println!("{} Stopping screensaver...", "→".yellow());
    
    // Kill all known screensaver processes
    for bin in SCREENSAVER_BINS {
        let _ = Command::new("pkill").arg(bin).status();
    }
    
    println!("  ✅ Screensavers stopped");
}

pub fn run(command: &ScreensaverCommands) {
    match command {
        ScreensaverCommands::List => list_screensavers(),
        ScreensaverCommands::Start { name } => start_screensaver(name),
        ScreensaverCommands::Stop => stop_screensaver(),
    }
}

/// Default action: launch a random screensaver
pub fn run_default() {
    use std::process::Command;
    
    println!("{}", "🎨  Launching Omara screensaver...".magenta());
    
    for bin in SCREENSAVER_BINS {
        if screensaver_exists(bin) {
            let _ = Command::new(bin).status();
            return;
        }
    }
    
    println!("{}", "⚠️  Could not find any Omara screensaver.".yellow());
    println!("   Install screensavers from omara-art/screensavers:");
    println!("   cargo install --path omara-art/screensavers");
}
