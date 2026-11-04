use std::collections::HashSet;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tracing::info;

use crate::api::Client;
use crate::config::Config;
use crate::error::Result;
use crate::run;

/// How long to let the server settle after a timeline frame before the
/// metadata is fetched. Analysis trails item creation by a few seconds.
const SETTLE: Duration = Duration::from_secs(3);

/// How long to wait before trying the socket again. A dropped connection
/// is followed by a full pass over the libraries, so reconnecting quickly
/// would only repeat work the next pass does anyway.
const RECONNECT_WAIT: Duration = Duration::from_secs(30 * 60);

/// Listen until the process is stopped. Each connection is preceded by a
/// pass that writes editions for movies still missing one, which also
/// covers whatever was scanned while the socket was down.
pub async fn listen(config: &Config, client: &Client) -> Result<()> {
    loop {
        catch_up(config, client).await;
        match session(config, client).await {
            Ok(()) => {}
            Err(err) => {
                info!(error = %err, wait_secs = RECONNECT_WAIT.as_secs(), "websocket disconnected");
                sleep(RECONNECT_WAIT).await;
            }
        }
    }
}

async fn catch_up(config: &Config, client: &Client) {
    match run::run(config, client, run::Mode::All).await {
        Ok(stats) => info!(
            written = stats.written,
            skipped = stats.skipped,
            failed = stats.failed,
            "caught up"
        ),
        Err(err) => info!(error = %err, "catch-up skipped"),
    }
}

async fn session(config: &Config, client: &Client) -> Result<()> {
    let url = websocket_url(client.base(), client.token());
    let request = url
        .into_client_request()
        .map_err(|err| crate::error::Error::WebSocket(err.to_string()))?;
    let (socket, _) = connect_async(request)
        .await
        .map_err(|err| crate::error::Error::WebSocket(err.to_string()))?;
    info!("websocket connected");
    let (_, mut incoming) = socket.split();
    let mut done: HashSet<String> = HashSet::new();
    while let Some(message) = incoming.next().await {
        let message = message.map_err(|err| crate::error::Error::WebSocket(err.to_string()))?;
        let text = match message {
            tokio_tungstenite::tungstenite::Message::Text(text) => text,
            tokio_tungstenite::tungstenite::Message::Ping(_)
            | tokio_tungstenite::tungstenite::Message::Pong(_) => continue,
            tokio_tungstenite::tungstenite::Message::Close(_) => break,
            _ => continue,
        };
        let Some(event) = parse_event(&text) else {
            continue;
        };
        if !event.is_new_movie() || done.contains(&event.rating_key) {
            continue;
        }
        sleep(SETTLE).await;
        match run::apply_one(config, client, &event.rating_key).await {
            // An empty edition means the media tree was not ready yet;
            // a later timeline frame for the same item retries.
            Ok(true) => {
                done.insert(event.rating_key);
            }
            Ok(false) => {}
            Err(err) => info!(rating_key = %event.rating_key, error = %err, "movie skipped"),
        }
    }
    Ok(())
}

pub fn websocket_url(base: &str, token: &str) -> String {
    let ws = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        format!("ws://{base}")
    };
    format!("{ws}/:/websockets/notifications?X-Plex-Token={token}")
}

/// A library timeline frame: how this server reports new and changed items.
#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub rating_key: String,
    pub metadata_state: String,
    pub state: i64,
}

impl Event {
    /// A movie reported as just created, or the final frame once its
    /// metadata is loaded. Either is a moment the edition may be writable.
    pub fn is_new_movie(&self) -> bool {
        self.metadata_state == "created" || self.state == 5
    }
}

/// The socket carries `NotificationContainer` frames; a new movie shows up
/// as a timeline entry with `itemID`, `type` 1, and a `metadataState`.
/// Verified against 1.43.5 by capturing frames during a library scan.
pub fn parse_event(text: &str) -> Option<Event> {
    let body: Value = serde_json::from_str(text).ok()?;
    let container = body.get("NotificationContainer")?;
    if container.get("type")?.as_str()? != "timeline" {
        return None;
    }
    let entry = container.get("TimelineEntry")?.as_array()?.first()?;
    if entry.get("identifier")?.as_str()? != "com.plexapp.plugins.library" {
        return None;
    }
    // metadata_type 1 is a movie; shows, seasons, and episodes are ignored.
    if entry.get("type")?.as_i64()? != 1 {
        return None;
    }
    let rating_key = match entry.get("itemID")? {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        _ => return None,
    };
    Some(Event {
        rating_key,
        metadata_state: entry
            .get("metadataState")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        state: entry.get("state").and_then(Value::as_i64).unwrap_or(-1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_switches_scheme_and_keeps_the_token() {
        assert_eq!(
            websocket_url("http://127.0.0.1:32400", "tok"),
            "ws://127.0.0.1:32400/:/websockets/notifications?X-Plex-Token=tok"
        );
        assert!(websocket_url("https://plex.example", "tok").starts_with("wss://"));
    }

    #[test]
    fn reads_a_created_movie_frame() {
        let text = r#"{"NotificationContainer":{"type":"timeline","size":1,"TimelineEntry":[{"identifier":"com.plexapp.plugins.library","sectionID":"2","itemID":"1029","type":1,"title":"","state":0,"metadataState":"created","updatedAt":1793782739}]}}"#;
        let event = parse_event(text).unwrap();
        assert_eq!(event.rating_key, "1029");
        assert!(event.is_new_movie());
    }

    #[test]
    fn reads_the_final_metadata_frame() {
        let text = r#"{"NotificationContainer":{"type":"timeline","size":1,"TimelineEntry":[{"identifier":"com.plexapp.plugins.library","sectionID":"2","itemID":"1029","type":1,"title":"Second Movie (2026)","state":5,"updatedAt":1793782739}]}}"#;
        let event = parse_event(text).unwrap();
        assert!(event.is_new_movie());
    }

    #[test]
    fn ignores_activity_frames_and_episodes() {
        let activity = r#"{"NotificationContainer":{"type":"activity","size":1,"ActivityNotification":[{"event":"started","uuid":"x","Activity":{"type":"library.update.section"}}]}}"#;
        assert!(parse_event(activity).is_none());
        let episode = r#"{"NotificationContainer":{"type":"timeline","size":1,"TimelineEntry":[{"identifier":"com.plexapp.plugins.library","sectionID":"1","itemID":"55","type":4,"state":0,"metadataState":"created"}]}}"#;
        assert!(parse_event(episode).is_none());
    }
}
