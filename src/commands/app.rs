use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[derive(Subcommand)]
pub enum AppCommands {
    /// List installed Omara applications
    List,

    /// Install an application
    Install {
        /// Package name to install
        package: String,
    },

    /// Remove an application
    Remove {
        /// Package name to remove
        package: String,
    },

    /// Search for applications
    Search {
        /// Search query
        query: String,
    },

    /// Reset an application to defaults
    Reset {
        /// Application to reset
        package: String,
    },
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// Check if a command exists
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Helper to recursively parse manifest files
fn parse_manifest_file(file_path: &Path, manifest_dir: &Path, visited: &mut HashSet<PathBuf>, apps: &mut Vec<String>) {
    let canonical = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(_) => file_path.to_path_buf(),
    };
    if !visited.insert(canonical) {
        return; // Prevent infinite recursion
    }

    if let Ok(content) = fs::read_to_string(file_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("@include ") {
                let include_path = trimmed["@include ".len()..].trim();
                let full_include_path = manifest_dir.join(include_path);
                parse_manifest_file(&full_include_path, manifest_dir, visited, apps);
            } else if !trimmed.starts_with('@') {
                apps.push(trimmed.to_string());
            }
        }
    }
}

/// Load app list from manifests
fn load_app_manifests() -> Vec<String> {
    let manifest_dir = crate::paths::get_component_path("omara-apps").join("manifests");
    
    let default_path = manifest_dir.join("default.txt");
    let mut apps = Vec::new();
    let mut visited = HashSet::new();
    
    if default_path.exists() {
        parse_manifest_file(&default_path, &manifest_dir, &mut visited, &mut apps);
        if !apps.is_empty() {
            return apps;
        }
    }
    
    // Fallback: return common Omara apps
    vec![
        "firefox".to_string(),
        "kitty".to_string(),
        "fish".to_string(),
        "neovim".to_string(),
        "quickshell".to_string(),
        "swaync".to_string(),
        "niri".to_string(),
        "fastfetch".to_string(),
        "htop".to_string(),
        "btop".to_string(),
    ]
}

/// List all Omara applications
fn list_apps() {
    println!("{}", "📦  Omara Applications".bold().cyan());
    println!();
    
    let apps = load_app_manifests();
    
    for app in &apps {
        if command_exists(app) {
            println!("  {}  {}", "✓".green(), app);
        } else {
            println!("  {}  {}", "✗".red(), app);
        }
    }
    
    println!();
    println!("  Total: {} apps", apps.len());
}

/// Install an application
fn install_app(package: &str) {
    println!("{} Installing {}...", "→".yellow(), package);
    
    // Try dnf first (will prompt for sudo password)
    let dnf_result = Command::new("sudo")
        .args(["dnf", "install", "-y", package])
        .status();
    
    if dnf_result.is_ok() {
        println!("  ✅ Installed via dnf");
        return;
    }
    
    // Try flatpak
    let flatpak_result = Command::new("flatpak")
        .args(["install", "-y", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    
    if flatpak_result.is_ok() {
        println!("  ✅ Installed via flatpak");
        return;
    }
    
    println!("  ❌ Failed to install {}", package);
    println!("  Try: sudo dnf search {}", package);
}

/// Remove an application
fn remove_app(package: &str) {
    println!("{} Removing {}...", "→".yellow(), package);
    
    // Check if installed via dnf
    let dnf_check = Command::new("dnf")
        .args(["list", "installed", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    
    if dnf_check.is_ok() {
        let result = Command::new("sudo")
            .args(["dnf", "remove", "-y", package])
            .status();
        if result.is_ok() {
            println!("  ✅ Removed via dnf");
            return;
        }
    }
    
    // Check if installed via flatpak
    let flatpak_check = Command::new("flatpak")
        .args(["list", "--app", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    
    if flatpak_check.is_ok() {
        let result = Command::new("flatpak")
            .args(["uninstall", "-y", package])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if result.is_ok() {
            println!("  ✅ Removed via flatpak");
            return;
        }
    }
    
    println!("  ❌ {} not found or failed to remove", package);
}

/// Search for applications
fn search_apps(query: &str) {
    println!("{}", format!("🔍  Searching for '{}'", query).bold().cyan());
    println!();
    
    let apps = load_app_manifests();
    let query_lower = query.to_lowercase();
    
    let matches: Vec<&String> = apps.iter()
        .filter(|app| app.to_lowercase().contains(&query_lower))
        .collect();
    
    if matches.is_empty() {
        println!("  No matches found in Omara apps.");
        println!("  Try: sudo dnf search {}", query);
        return;
    }
    
    for app in matches {
        if command_exists(app) {
            println!("  {}  {}", "✓".green(), app);
        } else {
            println!("  {}  {}", "○".bright_black(), app);
        }
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(entry.path(), dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn restore_config(src: &Path, target_dir_path: &str) -> std::io::Result<()> {
    let dst_str = expand_path(target_dir_path);
    let dst = Path::new(&dst_str);
    
    if src.exists() {
        if dst.exists() {
            let _ = fs::remove_dir_all(dst);
        }
        copy_dir_all(src, dst)?;
        println!("  Restored configuration to {}", dst.display());
    } else {
        println!("  ⚠️  Source configuration not found at {}", src.display());
    }
    Ok(())
}

/// Reset an application
fn reset_app(package: &str) {
    println!("{} Resetting {} to defaults...", "→".yellow(), package);
    
    let configs_dir = crate::paths::get_component_path("omara-core").join("configs");
    let de_dir = crate::paths::get_component_path("omara-de");
    
    // List of (repo_source, target_dest)
    let config_restores = match package {
        "kitty" => vec![
            (configs_dir.join("kitty"), "~/.config/kitty".to_string())
        ],
        "fish" => vec![
            (configs_dir.join("fish"), "~/.config/fish".to_string())
        ],
        "gh" => vec![
            (configs_dir.join("gh"), "~/.config/gh".to_string())
        ],
        "niri" => vec![
            (de_dir.join("niri"), "~/.config/niri".to_string()),
            (de_dir.join("niri"), "~/.config/omara/niri".to_string())
        ],
        "quickshell" => vec![
            (de_dir.join("niri").join("quickshell"), "~/.config/quickshell".to_string())
        ],
        "neovim" | "nvim" => {
            println!("  Removing Neovim local config...");
            let _ = fs::remove_dir_all(expand_path("~/.config/nvim"));
            let _ = fs::remove_dir_all(expand_path("~/.config/neovim"));
            vec![]
        }
        "swaync" => {
            println!("  Removing SwayNC local config...");
            let _ = fs::remove_dir_all(expand_path("~/.config/swaync"));
            vec![]
        }
        _ => vec![],
    };
    
    if config_restores.is_empty() && package != "neovim" && package != "nvim" && package != "swaync" {
        println!("  ⚠️  Don't know how to reset {}", package);
        return;
    }
    
    for (src, dst) in &config_restores {
        if let Err(e) = restore_config(src, dst) {
            println!("  ❌ Failed to restore config: {}", e);
        }
    }
    
    println!("  ✅ Reset complete. Restart {} to use defaults.", package);
}

pub fn run(command: &AppCommands) {
    match command {
        AppCommands::List => list_apps(),
        AppCommands::Install { package } => install_app(package),
        AppCommands::Remove { package } => remove_app(package),
        AppCommands::Search { query } => search_apps(query),
        AppCommands::Reset { package } => reset_app(package),
    }
}

/// Default action: list apps
pub fn run_default() {
    list_apps();
}
