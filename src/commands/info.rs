use colored::Colorize;
use std::process::Command;

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

/// Get system info
fn get_system_info() -> String {
    let mut info = String::new();
    
    // OS info
    if let Ok(output) = Command::new("cat").arg("/etc/os-release").output() {
        if let Ok(release) = String::from_utf8(output.stdout) {
            for line in release.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    info.push_str(&format!("OS: {}\n", line.trim()));
                    break;
                }
            }
        }
    }
    
    // Kernel
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        if let Ok(kernel) = String::from_utf8(output.stdout) {
            info.push_str(&format!("Kernel: {}\n", kernel.trim()));
        }
    }
    
    // CPU
    if let Ok(output) = Command::new("lscpu").output() {
        if let Ok(cpu) = String::from_utf8(output.stdout) {
            for line in cpu.lines() {
                if line.starts_with("Model name:") {
                    info.push_str(&format!("CPU: {}\n", line.trim()));
                    break;
                }
            }
        }
    }
    
    // Memory
    if let Ok(output) = Command::new("free").arg("-h").output() {
        if let Ok(mem) = String::from_utf8(output.stdout) {
            if let Some(line) = mem.lines().nth(1) {
                info.push_str(&format!("Memory: {}\n", line.trim()));
            }
        }
    }
    
    // GPU
    if command_exists("lspci") {
        if let Ok(output) = Command::new("lspci").arg("-v").output() {
            if let Ok(gpu) = String::from_utf8(output.stdout) {
                for line in gpu.lines() {
                    if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                        info.push_str(&format!("GPU: {}\n", line.trim()));
                        break;
                    }
                }
            }
        }
    }
    
    info
}

/// Get Omara version
fn get_omara_version() -> String {
    // Try to read version from omara-os
    let version_path = crate::paths::get_component_path("omara-os")
        .join("releases")
        .join("v0.1.0.toml");
    if let Ok(content) = std::fs::read_to_string(&version_path) {
        for line in content.lines() {
            if line.starts_with("version =") {
                return line.split('=').nth(1).unwrap_or("unknown").trim().trim_matches('"').to_string();
            }
        }
    }
    "0.1.0".to_string()
}

/// Get Omara component versions
fn get_component_versions() -> String {
    let mut versions = String::new();
    
    let components = [
        ("omara-configs", crate::paths::get_component_path("omara-core")),
        ("omara-cli", crate::paths::get_component_path("omara-cli")),
        ("omara-de", crate::paths::get_component_path("omara-de")),
        ("omara-apps", crate::paths::get_component_path("omara-apps")),
        ("omara-art", crate::paths::get_component_path("omara-art")),
        ("omara-os", crate::paths::get_component_path("omara-os")),
        ("omara-rpms", crate::paths::get_component_path("omara-rpms")),
    ];
    
    for (name, path) in &components {
        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("describe")
            .arg("--tags")
            .arg("--always")
            .output()
        {
            if let Ok(tag) = String::from_utf8(output.stdout) {
                versions.push_str(&format!("  {}: {}\n", name, tag.trim()));
            } else {
                versions.push_str(&format!("  {}: unknown\n", name));
            }
        } else {
            versions.push_str(&format!("  {}: not cloned\n", name));
        }
    }
    
    versions
}

pub fn run() {
    println!("{}", "ℹ️  Omara System Information".bold().cyan());
    println!();
    
    // Omara version
    println!("Omara Version: v{}", get_omara_version());
    println!();
    
    // Component versions
    println!("Components:");
    println!("{}", get_component_versions());
    
    // System info
    println!("System:");
    println!("{}", get_system_info());
    
    // CLI info
    println!("CLI:");
    println!("  Binary: omara");
    println!("  Config: ~/.config/omara/");
    println!("  Logs: ~/.local/share/omara/logs/");
}
