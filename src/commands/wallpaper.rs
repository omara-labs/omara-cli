use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::process::Command;

#[derive(Subcommand)]
pub enum WallpaperCommands {
    /// List available wallpapers
    List,

    /// Set wallpaper
    Set {
        /// Wallpaper name or path
        name: String,
    },

    /// Cycle to next wallpaper
    Next,
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// Get current wallpaper from swaybg or similar
fn get_current_wallpaper() -> Option<String> {
    // Check swaybg config
    let swaybg_config = expand_path("~/.config/swaybg/config");
    if let Ok(config) = fs::read_to_string(swaybg_config) {
        for line in config.lines() {
            if line.trim().starts_with("image=") {
                return Some(line.trim().trim_start_matches("image=").to_string());
            }
        }
    }
    None
}

/// Set wallpaper using swaybg
fn set_wallpaper_swaybg(path: &str) {
    let _ = Command::new("swaybg")
        .arg("-i")
        .arg(path)
        .status();
}

/// List available wallpapers
fn list_wallpapers() {
    println!("{}", "🖼️  Available Wallpapers".bold().cyan());
    println!();
    
    let wallpapers_dir = crate::paths::get_component_path("omara-art").join("wallpapers");
    
    if let Ok(entries) = fs::read_dir(&wallpapers_dir) {
        let mut count = 0;
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    println!("  • {}", name);
                    count += 1;
                }
            }
        }
        if count == 0 {
            println!("No wallpapers found in {}", wallpapers_dir.display());
        }
    } else {
        println!("Wallpapers directory not found: {}", wallpapers_dir.display());
    }
    
    println!();
}

/// Set wallpaper
fn set_wallpaper(name: &str) {
    println!("{}", format!("→ Setting wallpaper '{}'...", name).yellow());
    
    let wallpapers_dir = crate::paths::get_component_path("omara-art").join("wallpapers");
    let wallpaper_path = wallpapers_dir.join(name);
    
    // Check if it's a full path
    let path_to_use = if name.starts_with('/') {
        name.to_string()
    } else {
        wallpaper_path.to_string_lossy().into_owned()
    };
    
    if !fs::metadata(&path_to_use).is_ok() {
        println!("❌ Wallpaper '{}' not found", name);
        println!("Available:");
        list_wallpapers();
        return;
    }
    
    // Try swaybg (works with Niri and Hyprland)
    set_wallpaper_swaybg(&path_to_use);
    
    // Also try swww (alternative)
    let _ = Command::new("swww")
        .arg("img")
        .arg(&path_to_use)
        .status();
    
    println!("{}", "✅ Wallpaper set!".green());
}

/// Cycle to next wallpaper
fn next_wallpaper() {
    println!("{}", "🔄 Cycling to next wallpaper...".bold().cyan());
    
    let wallpapers_dir = crate::paths::get_component_path("omara-art").join("wallpapers");
    
    if let Ok(entries) = fs::read_dir(&wallpapers_dir) {
        let wallpapers: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|s| !s.starts_with('.'))
            .collect();
        
        if wallpapers.is_empty() {
            println!("No wallpapers found");
            return;
        }
        
        // Find current and pick next
        if let Some(current) = get_current_wallpaper() {
            if let Some(pos) = wallpapers.iter().position(|w| *w == current) {
                let next_pos = (pos + 1) % wallpapers.len();
                set_wallpaper(&wallpapers[next_pos]);
                return;
            }
        }
        
        // No current found, use first
        set_wallpaper(&wallpapers[0]);
    }
}

pub fn run(command: &WallpaperCommands) {
    match command {
        WallpaperCommands::List => list_wallpapers(),
        WallpaperCommands::Set { name } => set_wallpaper(name),
        WallpaperCommands::Next => next_wallpaper(),
    }
}

/// Default action: list wallpapers
pub fn run_default() {
    list_wallpapers();
}
