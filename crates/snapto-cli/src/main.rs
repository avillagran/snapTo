use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod output;
mod progress;

use commands::{config, history, upload, watch};

#[derive(Parser)]
#[command(name = "snapto")]
#[command(about = "Upload images from clipboard to remote servers", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload image from clipboard
    Upload {
        /// Override destination (use configured default if not specified)
        #[arg(short, long)]
        destination: Option<String>,

        /// Custom filename (uses template if not specified)
        #[arg(short, long)]
        filename: Option<String>,
    },

    /// Watch clipboard for images and auto-upload
    Watch {
        /// Interval in milliseconds to check clipboard
        #[arg(short, long, default_value = "500")]
        interval: u64,

        /// Destination to upload to
        #[arg(short, long)]
        destination: Option<String>,
    },

    /// Show or edit configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Show upload history
    History {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Show full details
        #[arg(short = 'f', long)]
        full: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration in $EDITOR
    Edit,
    /// Show config file path
    Path,
    /// Initialize default configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "snapto=debug,snapto_core=debug"
    } else {
        "snapto=info,snapto_core=info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Execute command
    let result = match cli.command {
        Commands::Upload {
            destination,
            filename,
        } => upload::execute(destination, filename).await,

        Commands::Watch {
            interval,
            destination,
        } => watch::execute(interval, destination).await,

        Commands::Config { action } => {
            let action = action.unwrap_or(ConfigAction::Show);
            match action {
                ConfigAction::Show => config::show().await,
                ConfigAction::Edit => config::edit().await,
                ConfigAction::Path => config::path().await,
                ConfigAction::Init => config::init().await,
            }
        }

        Commands::History { limit, full } => history::execute(limit, full).await,
    };

    if let Err(e) = result {
        output::error(&format!("Error: {}", e));
        std::process::exit(1);
    }

    Ok(())
}
