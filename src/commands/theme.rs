use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum ThemeCommands {
    /// List available themes
    List,

    /// Apply a theme
    Set {
        /// Theme name to apply
        name: String,
    },

    /// Show current theme
    Current,
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// List available themes
fn list_themes() {
    println!("{}", "🎨  Available Themes".bold().cyan());
    println!();
    
    let themes_dir = crate::paths::get_component_path("omara-art").join("themes");
    
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                println!("  • {}", name);
            }
        }
    } else {
        // Fallback: show known themes
        println!("  • catppuccin-mocha");
        println!("  • gruvbox-dark");
        println!("  • nord");
        println!("  • tokyo-night");
        println!("  • rose-pine");
    }
    
    println!();
    println!("  Themes directory: {}", themes_dir.display());
}

/// Run theme hooks from user and system directories
fn run_theme_hooks(theme_name: &str, active_path: &Path) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
    let user_hooks_dir = Path::new(&home).join(".config").join("omara").join("theme.d");
    let system_hooks_dir = PathBuf::from("/etc/omara/theme.d");

    println!("  🏃 Running theme hooks...");

    // Create user hooks directory if it doesn't exist
    let _ = fs::create_dir_all(&user_hooks_dir);

    let mut hooks_run = 0;

    for hooks_dir in &[user_hooks_dir, system_hooks_dir] {
        if let Ok(entries) = fs::read_dir(hooks_dir) {
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();

            for path in paths {
                if path.is_file() {
                    // Check if file is executable on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        if let Ok(meta) = path.metadata() {
                            let mode = meta.mode();
                            let is_executable = mode & 0o111 != 0;
                            if !is_executable {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        println!("    → Executing hook: {}", file_name);
                        
                        let result = Command::new(&path)
                            .arg(theme_name)
                            .arg(active_path)
                            .status();

                        match result {
                            Ok(status) => {
                                if status.success() {
                                    hooks_run += 1;
                                } else {
                                    eprintln!("      ⚠️  Hook {} exited with error status: {:?}", file_name, status.code());
                                }
                            }
                            Err(e) => {
                                eprintln!("      ❌ Failed to execute hook {}: {}", file_name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    if hooks_run > 0 {
        println!("  ✅ Executed {} theme hooks.", hooks_run);
    } else {
        println!("  ℹ️  No theme hooks executed (add executable scripts to ~/.config/omara/theme.d/)");
    }
}

/// Set/apply a theme
fn set_theme(name: &str) {
    println!("{} Applying theme '{}'...", "→".yellow(), name);
    
    let themes_dir = crate::paths::get_component_path("omara-art").join("themes");
    let theme_path = themes_dir.join(name);
    
    if !theme_path.exists() {
        println!("  ❌ Theme '{}' not found", name.red());
        println!("  Available themes:");
        list_themes();
        return;
    }
    
    // Write theme name directly to configuration file
    let config_dir = expand_path("~/.config/omara");
    let _ = fs::create_dir_all(&config_dir);
    
    let theme_file = Path::new(&config_dir).join("active_theme.txt");
    if let Err(e) = fs::write(&theme_file, name) {
        println!("  ⚠️  Could not write active theme file: {}", e);
        return;
    }
    
    println!("  ✅ Active theme configuration updated!");

    // Run Option C script hooks, passing theme name and direct path to theme folder
    run_theme_hooks(name, &theme_path);
    
    println!("  ✅ Theme application complete!");
}

/// Show current theme
fn show_current_theme() {
    println!("{}", "🎨  Current Theme".bold().cyan());
    println!();
    
    let theme_str = expand_path("~/.config/omara/active_theme.txt");
    let theme_file = Path::new(&theme_str);
    if theme_file.exists() {
        if let Ok(name) = fs::read_to_string(&theme_file) {
            println!("  Current: {}", name.trim().yellow());
            return;
        }
    }
    
    println!("  No theme set");
    println!("  Set a theme: omara theme set <name>");
}

pub fn run(command: &ThemeCommands) {
    match command {
        ThemeCommands::List => list_themes(),
        ThemeCommands::Set { name } => set_theme(name),
        ThemeCommands::Current => show_current_theme(),
    }
}

/// Default action: list themes
pub fn run_default() {
    list_themes();
}
