use clap::Subcommand;
use colored::Colorize;
use std::fs;

const LOG_DIR: &str = "~/.local/share/omara/logs";
const LOG_FILE: &str = "~/.local/share/omara/logs/omara.log";

#[derive(Subcommand)]
pub enum LogCommands {
    /// Show Omara logs
    Show,

    /// Clear Omara logs
    Clear,
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// Show logs
fn show_logs() {
    println!("{}", "📜  Omara Logs".bold().cyan());
    println!();
    
    let log_path = expand_path(LOG_FILE);
    
    if let Ok(content) = fs::read_to_string(&log_path) {
        if content.is_empty() {
            println!("No logs found");
        } else {
            println!("{}", content);
        }
    } else {
        println!("No logs found at {}", log_path);
    }
    
    println!();
    println!("Log file: {}", log_path);
}

/// Clear logs
fn clear_logs() {
    println!("{}", "Clearing Omara logs...".yellow());
    
    let log_dir = expand_path(LOG_DIR);
    let log_path = expand_path(LOG_FILE);
    
    if fs::metadata(&log_path).is_ok() {
        fs::remove_file(&log_path).ok();
        println!("✅ Logs cleared");
    } else {
        println!("ℹ️  No logs to clear");
    }
    
    // Also clear old logs
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".log") && name != "omara.log" {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    
    println!("✅ All logs cleared");
}

pub fn run(command: &LogCommands) {
    match command {
        LogCommands::Show => show_logs(),
        LogCommands::Clear => clear_logs(),
    }
}

/// Default action: show logs
pub fn run_default() {
    show_logs();
}
