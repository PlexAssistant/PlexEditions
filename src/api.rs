use std::time::Duration;

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::error::{Error, HTTP_TIMEOUT, Result};
use crate::model::{self, Library, Movie};

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base: String,
    token: String,
}

impl Client {
    pub fn new(base: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Plex-Token",
            HeaderValue::from_str(token).map_err(|err| Error::Config(err.to_string()))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .connect_timeout(Duration::from_secs(10))
            .default_headers(headers)
            .build()?;
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_string(),
            token: token.to_string(),
        })
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub async fn friendly_name(&self) -> Result<String> {
        let body = self.get("").await?;
        model::friendly_name(&body).ok_or_else(|| Error::Decode("missing friendlyName".into()))
    }

    pub async fn libraries(&self) -> Result<Vec<Library>> {
        let body = self.get("/library/sections").await?;
        Ok(model::libraries(&body))
    }

    pub async fn movies(&self, library: &Library) -> Result<Vec<Movie>> {
        let body = self
            .get(&format!("/library/sections/{}/all", library.key))
            .await?;
        Ok(model::movies_from_section(&body, &library.title))
    }

    pub async fn movie(&self, rating_key: &str, library: &str) -> Result<Option<Movie>> {
        let body = self.get(&format!("/library/metadata/{rating_key}")).await?;
        Ok(model::movie_from_metadata(&body, library))
    }

    pub async fn set_edition(&self, rating_key: &str, title: &str) -> Result<()> {
        self.put_edition(rating_key, title, "1").await
    }

    pub async fn clear_edition(&self, rating_key: &str) -> Result<()> {
        self.put_edition(rating_key, "", "0").await
    }

    async fn put_edition(&self, rating_key: &str, title: &str, locked: &str) -> Result<()> {
        let url = format!(
            "{}/library/metadata/{rating_key}?type=1&id={}&{}&{}",
            self.base,
            encode(rating_key),
            query("editionTitle.value", title),
            query("editionTitle.locked", locked),
        );
        let response = self.http.put(&url).send().await?;
        if !response.status().is_success() {
            return Err(Error::Plex {
                status: response.status(),
                url,
            });
        }
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{path}", self.base);
        let response = self.http.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(Error::Plex {
                status: response.status(),
                url,
            });
        }
        let bytes = response.bytes().await?;
        model::parse_body(&bytes)
    }
}

fn query(key: &str, value: &str) -> String {
    format!("{}={}", encode(key), encode(value))
}

fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
