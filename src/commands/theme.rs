use clap::Subcommand;
use colored::Colorize;
use std::fs;

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
    
    // Symlink theme to config directory
    let config_dir = expand_path("~/.config/omara/theme");
    let _ = fs::create_dir_all(&config_dir);
    
    let target = format!("{}/{}", config_dir, name);
    
    // Remove existing
    let _ = fs::remove_file(&target);
    let _ = fs::remove_dir_all(&target);
    
    // Create symlink
    if let Err(e) = std::os::unix::fs::symlink(&theme_path, &target) {
        println!("  ⚠️  Could not symlink: {}", e);
        println!("  Manually link: ln -s {} ~/.config/omara/theme/{}", theme_path.display(), name);
        return;
    }
    
    println!("  ✅ Theme applied!");
    println!("  Restart your applications to see changes.");
}

/// Show current theme
fn show_current_theme() {
    println!("{}", "🎨  Current Theme".bold().cyan());
    println!();
    
    let config_dir = expand_path("~/.config/omara/theme");
    
    if let Ok(entries) = fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                println!("  Current: {}", name);
                return;
            }
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
