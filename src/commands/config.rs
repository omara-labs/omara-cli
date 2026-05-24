use clap::Subcommand;
use colored::Colorize;
use std::fs;

const CONFIG_DIR: &str = "~/.config/omara";
const CONFIG_FILE: &str = "~/.config/omara/cli.toml";

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key to get
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key to set
        key: String,
        /// Value to set
        value: String,
    },

    /// List all configuration
    List,

    /// Reset configuration to defaults
    Reset,
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// Read config file into a HashMap
fn read_config() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    
    let config_path = expand_path(CONFIG_FILE);
    
    if let Ok(content) = fs::read_to_string(&config_path) {
        let mut config = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                config.insert(
                    key.trim().to_string(),
                    value.trim().to_string()
                );
            }
        }
        return config;
    }
    
    std::collections::HashMap::new()
}

/// Write config HashMap to file
fn write_config(config: &std::collections::HashMap<String, String>) -> Result<(), String> {
    let config_path = expand_path(CONFIG_FILE);
    let parent = expand_path(CONFIG_DIR);
    
    fs::create_dir_all(&parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let mut content = String::new();
    content.push_str("# Omara CLI Configuration\n\n");
    
    for (key, value) in config {
        content.push_str(&format!("{} = {}\n", key, value));
    }
    
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

/// Get a configuration value
fn get_config(key: &str) -> Result<String, String> {
    let config = read_config();
    config.get(key)
        .cloned()
        .ok_or_else(|| format!("Configuration key '{}' not found", key))
}

/// Set a configuration value
fn set_config(key: &str, value: &str) -> Result<(), String> {
    let mut config = read_config();
    config.insert(key.to_string(), value.to_string());
    write_config(&config)
}

/// List all configuration
fn list_config() {
    println!("{}", "⚙️  Omara CLI Configuration".bold().cyan());
    println!();
    
    let config = read_config();
    
    if config.is_empty() {
        println!("  No configuration set");
        println!("  Config file: {}", expand_path(CONFIG_FILE));
        return;
    }
    
    for (key, value) in &config {
        println!("  {} = {}", key.bold(), value);
    }
    
    println!();
    println!("  Config file: {}", expand_path(CONFIG_FILE));
}

/// Reset configuration
fn reset_config() {
    let config_path = expand_path(CONFIG_FILE);
    
    if fs::metadata(&config_path).is_ok() {
        fs::remove_file(&config_path).ok();
        println!("  ✅ Configuration reset to defaults");
    } else {
        println!("  ℹ️  No custom configuration to reset");
    }
}

/// Get a config value
fn cmd_get(key: &str) {
    match get_config(key) {
        Ok(value) => {
            println!("{}: {}", key.bold(), value);
        }
        Err(e) => {
            println!("  ❌  {}", e.red());
        }
    }
}

/// Set a config value
fn cmd_set(key: &str, value: &str) {
    if let Err(e) = set_config(key, value) {
        println!("  ❌  {}", e.red());
    } else {
        println!("  ✅ {} = {}", key, value);
    }
}

pub fn run(command: &ConfigCommands) {
    match command {
        ConfigCommands::Get { key } => cmd_get(key),
        ConfigCommands::Set { key, value } => cmd_set(key, value),
        ConfigCommands::List => list_config(),
        ConfigCommands::Reset => reset_config(),
    }
}

/// Default action: list config
pub fn run_default() {
    list_config();
}
