use std::time::Duration;

use tokio::time::sleep;
use tracing::info;

use crate::api;
use crate::config::{self, Config};
use crate::error::Result;

/// How often `new` retries while the server is not ready yet. A server
/// that has not been claimed stays that way for a while, so retrying
/// quickly would only repeat the same failure.
const STARTUP_WAIT: Duration = Duration::from_secs(30 * 60);

/// Load the config and connect for the `new` service, retrying while the
/// token cannot be read yet or the server refuses the connection. Returns
/// the config, a client, and the server's friendly name. `default_url` is
/// the `PLEX_URL` fallback for embedders; an empty string keeps the
/// variable required.
pub async fn await_ready(default_url: &str) -> Result<(Config, api::Client, String)> {
    let config = config_when_ready(default_url).await?;
    let client = api::Client::new(&config.plex_url, &config.plex_token)?;
    let name = server_when_ready(&client).await?;
    Ok((config, client, name))
}

/// Load the config, retrying while the token cannot be read yet. The Plex
/// data directory gains `PlexOnlineToken` only after the server is claimed.
async fn config_when_ready(default_url: &str) -> Result<Config> {
    loop {
        match config::Config::from_env_with_url(default_url) {
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
