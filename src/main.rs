use clap::{Parser, Subcommand};

mod commands;
mod paths;

#[derive(Parser)]
#[command(
    name = "omara",
    version,
    about = "The official Omara command-line tool — clean, fast, yours.",
    long_about = "Manage updates, screensavers, and more.\nPart of the Omara project.",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update system packages (dnf + flatpak)
    Update,

    /// Launch a screensaver (default: random)
    Screensaver {
        #[command(subcommand)]
        action: Option<commands::screensaver::ScreensaverCommands>,
    },

    /// Run system health checks (default: full)
    Doctor {
        #[command(subcommand)]
        action: Option<commands::doctor::DoctorCommands>,
    },

    /// Manage applications
    App {
        #[command(subcommand)]
        action: Option<commands::app::AppCommands>,
    },

    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: Option<commands::theme::ThemeCommands>,
    },

    /// Manage wallpapers
    Wallpaper {
        #[command(subcommand)]
        action: Option<commands::wallpaper::WallpaperCommands>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: Option<commands::config::ConfigCommands>,
    },

    /// View and manage logs
    Log {
        #[command(subcommand)]
        action: Option<commands::log::LogCommands>,
    },

    /// Show system information
    Info,

    /// Manage the Omara OS installation and state
    #[command(alias = "system")]
    Os {
        #[command(subcommand)]
        action: commands::os::OsCommands,
    },

    /// Get help or ask questions using AI (e.g., omara help "how do I install firefox")
    Help { question: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Update => commands::update::run(),
        Commands::Screensaver { action } => {
            match action {
                Some(a) => commands::screensaver::run(a),
                None => commands::screensaver::run_default(),
            }
        }
        Commands::Doctor { action } => {
            match action {
                Some(a) => commands::doctor::run(a),
                None => commands::doctor::run_default(),
            }
        }
        Commands::App { action } => {
            match action {
                Some(a) => commands::app::run(a),
                None => commands::app::run_default(),
            }
        }
        Commands::Theme { action } => {
            match action {
                Some(a) => commands::theme::run(a),
                None => commands::theme::run_default(),
            }
        }
        Commands::Wallpaper { action } => {
            match action {
                Some(a) => commands::wallpaper::run(a),
                None => commands::wallpaper::run_default(),
            }
        }
        Commands::Config { action } => {
            match action {
                Some(a) => commands::config::run(a),
                None => commands::config::run_default(),
            }
        }
        Commands::Log { action } => {
            match action {
                Some(a) => commands::log::run(a),
                None => commands::log::run_default(),
            }
        }
        Commands::Info => commands::info::run(),
        Commands::Os { action } => commands::os::run(action),
        Commands::Help { question } => commands::help::run(question.clone()),
    }
}
