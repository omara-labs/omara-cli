//! AI-powered help system using Ollama
//!
//! Usage:
//!   omara help                    # Show CLI help (delegates to clap)
//!   omara help "how do I install firefox"  # Ask AI for help

use colored::Colorize;
use std::io::{self, Write};
use std::process::Command;

/// Model candidates: (internal_name, display_name, size_in_mb)
const MODEL_CANDIDATES: &[(&str, &str, u64)] = &[
    ("gemma4:e4b", "gemma4:e4b", 10_000),
    ("qwen3.5:4.0b", "qwen3.5:4.0b", 2_400),
    ("qwen3.5:1.8b", "qwen3.5:1.8b", 1_100),
    ("qwen3.5:0.6b", "qwen3.5:0.6b", 400),
];

/// Check if ollama binary exists
fn ollama_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Check if ollama server is running
fn ollama_running() -> bool {
    Command::new("ollama")
        .arg("ps")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Check if any models are downloaded
fn has_models() -> bool {
    let output = Command::new("ollama")
        .arg("list")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();
    output.is_ok() && !output.unwrap().stdout.is_empty()
}

/// Full ollama readiness check
fn ollama_ready() -> bool {
    ollama_installed() && ollama_running() && has_models()
}

/// Detect GPU type
fn detect_gpu() -> Option<String> {
    // NVIDIA
    if Command::new("nvidia-smi").status().is_ok() {
        return Some("NVIDIA".to_string());
    }

    // AMD or Intel via lspci
    let lspci = Command::new("lspci").output().ok()?;
    let output = String::from_utf8(lspci.stdout).ok()?;

    if output.contains("AMD") || output.contains("Radeon") {
        return Some("AMD".to_string());
    }

    if output.contains("Intel") && output.contains("Graphics") {
        return Some("Intel".to_string());
    }

    None
}

/// Expand a path with tilde
fn expand_path(path: &str) -> String {
    shellexpand::tilde(path).to_string()
}

/// Get available disk space in bytes for a path
fn get_disk_space(path: &str) -> Result<u64, String> {
    let expanded = expand_path(path);
    let output = Command::new("df")
        .arg("--output=avail")
        .arg(&expanded)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to check disk: {}", e)),
    };

    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Err("Invalid disk output encoding".to_string()),
    };

    // df output may have header line, skip to second line
    let avail_kb: u64 = output_str
        .lines()
        .nth(1)  // Skip header line
        .ok_or("Unexpected df output format")?
        .trim()
        .parse()
        .map_err(|_| "Failed to parse disk space".to_string())?;

    Ok(avail_kb * 1024) // Convert KB to bytes
}

/// Get model size in GB for display
fn model_size_gb(name: &str) -> u64 {
    MODEL_CANDIDATES
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, _, size_mb)| size_mb / 1024)
        .unwrap_or(0)
}

/// Run a shell command and return Result
fn run_command(cmd: &str) -> Result<(), String> {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();

    let status = Command::new(program).args(args).status();

    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(_) => Err(format!("Command failed: {}", cmd)),
        Err(e) => Err(format!("Failed to run '{}': {}", cmd, e)),
    }
}

/// Select best model that fits available disk space
fn select_model() -> Result<(&'static str, &'static str), String> {
    let free_space_bytes = get_disk_space("/")?;
    let free_space_mb = free_space_bytes / 1024 / 1024;

    MODEL_CANDIDATES
        .iter()
        .find(|(_, _, required_mb)| free_space_mb >= *required_mb)
        .map(|(name, display, _)| (*name, *display))
        .ok_or_else(|| "No model fits available disk space".to_string())
}

/// Ensure Ollama is installed, running, and has a model
fn ensure_ollama() -> Result<(), String> {
    // Already ready?
    if ollama_ready() {
        return Ok(());
    }

    // Check GPU
    let gpu = detect_gpu().ok_or_else(|| {
        "❌ Ollama requires GPU support.\n   Manual install: https://ollama.com".to_string()
    })?;

    // Select model based on disk space
    let (model_name, model_display) = select_model()?;
    let model_size = model_size_gb(model_name);

    // Get disk space for display
    let free_space_bytes = get_disk_space("/")?;
    let free_space_gb = free_space_bytes / 1024 / 1024 / 1024;

    // Ask user
    println!();
    println!("{}", "🤖  Ollama Setup Required".bold().cyan());
    println!("   GPU: {}", gpu);
    println!("   Free disk: {} GB", free_space_gb);
    println!();
    println!("   Omara will:");
    println!("   • Install Ollama");
    println!("   • Enable systemd service");
    println!("   • Download {} (~{} GB)", model_display, model_size);
    println!();
    print!("   Proceed? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| "Failed to read input".to_string())?;

    if input.trim().to_lowercase() != "y" {
        return Err("Cancelled by user".to_string());
    }

    // Install Ollama
    println!("\n   → Installing Ollama...");
    run_command("curl -fsSL https://ollama.com/install.sh | sh")?;

    // Enable systemd
    println!("   → Enabling systemd service...");
    run_command("sudo systemctl enable ollama --now")?;

    // Pull model
    println!("   → Downloading {} (this may take a while)...", model_display);
    run_command(&format!("ollama pull {}", model_name))?;

    Ok(())
}

/// Get the currently available model name
fn get_model_name() -> Result<String, String> {
    for (name, _, _) in MODEL_CANDIDATES {
        let output = Command::new("ollama")
            .arg("list")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output();
        if let Ok(output) = output {
            let list = String::from_utf8_lossy(&output.stdout);
            if list.contains(name) {
                return Ok(name.to_string());
            }
        }
    }
    Err("No Ollama models found".to_string())
}

/// Send prompt to Ollama and get response
fn ask_ollama(prompt: &str) -> Result<String, String> {
    use std::process::Stdio;
    
    let model = get_model_name()?;
    
    let child = Command::new("ollama")
        .arg("run")
        .arg(&model)
        .arg(prompt)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Ollama: {}", e))?;

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read Ollama output: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ollama error: {}", stderr));
    }

    let response = String::from_utf8(output.stdout)
        .map_err(|_| "Invalid Ollama response".to_string())?;

    Ok(response.trim().to_string())
}

/// Main run function
pub fn run(question: Option<String>) {
    match question {
        // No question - show standard CLI help
        None => {
            println!();
            println!("{}", "Omara CLI - Clean. Professional. A little fun.".bold().cyan());
            println!();
            println!("Usage: omara <COMMAND>");
            println!();
            println!("Commands:");
            println!("  update        Update system packages (dnf + flatpak)");
            println!("  de           Manage desktop environments");
            println!("  de list      List available desktop environments");
            println!("  de current   Show current desktop environment");
            println!("  de switch    Switch desktop environment");
            println!("  screensaver  Launch a random Omara screensaver");
            println!("  doctor       Run system health checks");
            println!("  help         Get help or ask questions (omara help \"your question\")");
            println!();
            println!("For more help on a command: omara <command> --help");
        }
        // Question provided - use AI
        Some(q) => {
            // Ensure Ollama is set up
            if let Err(e) = ensure_ollama() {
                println!("{}", e);
                println!("\n   Run 'ollama serve' and 'ollama pull <model>' manually.");
                return;
            }

            println!("{}", "🤖  Omara Help AI".bold().cyan());
            println!("   Question: {}", q.italic());
            println!();

            match ask_ollama(&q) {
                Ok(response) => {
                    println!("{}", response);
                }
                Err(e) => {
                    println!("{}", format!("❌  {}", e).red());
                }
            }
        }
    }
}
