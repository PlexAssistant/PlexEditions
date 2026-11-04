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
use crate::run::{self, backoff};

/// Listen until the process is stopped. A dropped socket reconnects with backoff.
pub async fn listen(config: &Config, client: &Client) -> Result<()> {
    let mut attempt = 0_u32;
    loop {
        match session(config, client).await {
            Ok(()) => attempt = 0,
            Err(err) => {
                let wait = backoff(attempt);
                info!(error = %err, wait_secs = wait.as_secs(), "websocket disconnected");
                sleep(wait).await;
                attempt = attempt.saturating_add(1);
            }
        }
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
    while let Some(message) = incoming.next().await {
        let message = message.map_err(|err| crate::error::Error::WebSocket(err.to_string()))?;
        let text = match message {
            tokio_tungstenite::tungstenite::Message::Text(text) => text,
            tokio_tungstenite::tungstenite::Message::Ping(_)
            | tokio_tungstenite::tungstenite::Message::Pong(_) => continue,
            tokio_tungstenite::tungstenite::Message::Close(_) => break,
            _ => continue,
        };
        if let Some(event) = parse_event(text.as_str()) {
            if event.kind != "library.new" || event.item_type != "movie" {
                continue;
            }
            if config.skip_libraries.contains(&event.library) {
                continue;
            }
            // A new item can arrive before its media tree is ready.
            sleep(Duration::from_secs(2)).await;
            if let Err(err) =
                run::apply_one(config, client, &event.rating_key, &event.library).await
            {
                info!(rating_key = %event.rating_key, error = %err, "new movie skipped");
            }
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

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub kind: String,
    pub rating_key: String,
    pub item_type: String,
    pub library: String,
}

/// Plex notification frames carry `NotificationContainer`, not the webhook shape.
pub fn parse_event(text: &str) -> Option<Event> {
    let body: Value = serde_json::from_str(text).ok()?;
    let container = body.get("NotificationContainer")?;
    let kind = container.get("type")?.as_str()?.to_string();
    let entry = container
        .get("PlaySessionStateNotification")
        .or_else(|| container.get("ActivityNotification"))
        .or_else(|| container.get("TimelineEntry"))
        .and_then(Value::as_array)
        .and_then(|entries| entries.first())
        .cloned()
        .unwrap_or(Value::Null);
    let rating_key = json_id(container.get("ratingKey"))
        .or_else(|| json_id(entry.get("ratingKey")))
        .unwrap_or_default();
    let item_type = container
        .get("itemType")
        .or_else(|| entry.get("type"))
        .or_else(|| entry.get("metadataType"))
        .and_then(Value::as_str)
        .unwrap_or("movie")
        .to_string();
    let library = container
        .get("librarySectionTitle")
        .or_else(|| entry.get("librarySectionTitle"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some(Event {
        kind,
        rating_key,
        item_type,
        library,
    })
}

fn json_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
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
    fn reads_a_library_new_frame() {
        let text = r#"{"NotificationContainer":{"type":"timeline","size":1,"TimelineEntry":[{"identifier":"com.plexapp.plugins.library","state":0,"itemID":42,"type":1,"title":"Example"}]}}"#;
        let event = parse_event(text).unwrap();
        assert_eq!(event.kind, "timeline");
    }

    #[test]
    fn reads_rating_key_when_present() {
        let text = r#"{"NotificationContainer":{"type":"library.new","ratingKey":"42","itemType":"movie","librarySectionTitle":"Movies"}}"#;
        let event = parse_event(text).unwrap();
        assert_eq!(
            event,
            Event {
                kind: "library.new".into(),
                rating_key: "42".into(),
                item_type: "movie".into(),
                library: "Movies".into(),
            }
        );
    }
}
