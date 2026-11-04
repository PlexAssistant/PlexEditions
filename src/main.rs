mod api;
mod config;
mod error;
mod i18n;
mod model;
mod modules;
mod run;
mod ws;

use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use tokio::time::sleep;
use tracing::info;

use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Command {
    /// Write editions for movies that do not have one, then exit.
    All,
    /// Listen for new movies and write an edition for each.
    New,
    /// Clear editions in movie libraries, then exit.
    Reset,
}

#[derive(Parser)]
#[command(
    name = "editions",
    about = "Write movie edition titles on a Plex server"
)]
struct Cli {
    #[arg(value_enum)]
    command: Command,
    /// Print the edition that would be written. Do not change the server.
    #[arg(long)]
    dry_run: bool,
}

/// How often `new` retries while the server is not ready yet. A server
/// that has not been claimed stays that way for a while, so retrying
/// quickly would only repeat the same failure.
const STARTUP_WAIT: Duration = Duration::from_secs(30 * 60);

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .without_time()
        .init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    // `new` is a long-running service: Plex or its token may not be ready
    // when the container starts. Retry slowly instead of exiting. `all` and
    // `reset` run once, so they still fail immediately.
    let mut config = if matches!(cli.command, Command::New) {
        config_when_ready().await?
    } else {
        Config::from_env()?
    };
    config.dry_run = cli.dry_run;
    let client = api::Client::new(&config.plex_url, &config.plex_token)?;
    let name = if matches!(cli.command, Command::New) {
        server_when_ready(&client).await?
    } else {
        client.friendly_name().await?
    };
    info!("connected to {name}");
    match cli.command {
        Command::All => {
            let stats = run::run(&config, &client, run::Mode::All).await?;
            info!(
                written = stats.written,
                skipped = stats.skipped,
                failed = stats.failed,
                "done"
            );
        }
        Command::Reset => {
            let stats = run::run(&config, &client, run::Mode::Reset).await?;
            info!(
                cleared = stats.written,
                skipped = stats.skipped,
                failed = stats.failed,
                "done"
            );
        }
        // The listener writes editions for movies that do not have one
        // before each connection, so a restart also covers the backlog.
        Command::New => ws::listen(&config, &client).await?,
    }
    Ok(())
}

/// Load the config, retrying while the token cannot be read yet. The Plex
/// data directory gains `PlexOnlineToken` only after the server is claimed.
async fn config_when_ready() -> Result<Config> {
    loop {
        match Config::from_env() {
            Ok(config) => return Ok(config),
            Err(err) => {
                info!(error = %err, wait_secs = STARTUP_WAIT.as_secs(), "config not ready");
                sleep(STARTUP_WAIT).await;
            }
        }
    }
}

/// Ask the server its name, retrying while it refuses the connection.
async fn server_when_ready(client: &api::Client) -> Result<String> {
    loop {
        match client.friendly_name().await {
            Ok(name) => return Ok(name),
            Err(err) => {
                info!(error = %err, wait_secs = STARTUP_WAIT.as_secs(), "plex not ready");
                sleep(STARTUP_WAIT).await;
            }
        }
    }
}
