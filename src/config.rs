use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::i18n::Language;

/// Labels the user can put in `EDITION_MODULES`. English ids are accepted too.
const MODULE_ALIASES: &[(&str, &str)] = &[
    ("source", "source"),
    ("片源版本", "source"),
    ("cut", "cut"),
    ("剪辑版本", "cut"),
    ("release", "release"),
    ("发行版本", "release"),
    ("resolution", "resolution"),
    ("分辨率", "resolution"),
    ("dynamicrange", "dynamicrange"),
    ("动态范围", "dynamicrange"),
    ("videocodec", "videocodec"),
    ("视频编码", "videocodec"),
    ("framerate", "framerate"),
    ("帧率", "framerate"),
    ("audiocodec", "audiocodec"),
    ("音频编码", "audiocodec"),
    ("bitrate", "bitrate"),
    ("比特率", "bitrate"),
    ("size", "size"),
    ("大小", "size"),
    ("country", "country"),
    ("国家", "country"),
    ("contentrating", "contentrating"),
    ("内容分级", "contentrating"),
    ("rating", "rating"),
    ("评分", "rating"),
    ("duration", "duration"),
    ("时长", "duration"),
];

/// Default order when `EDITION_MODULES` is unset.
pub const DEFAULT_MODULES: &[&str] = &[
    "source",
    "resolution",
    "audiocodec",
    "videocodec",
    "framerate",
    "bitrate",
    "duration",
    "rating",
    "cut",
    "release",
    "dynamicrange",
    "country",
    "contentrating",
    "size",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub plex_url: String,
    pub plex_token: String,
    pub modules: Vec<String>,
    pub language: Language,
    pub skip_libraries: HashSet<String>,
    pub separator: String,
    pub dry_run: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let plex_url = env::var("PLEX_URL")
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .map_err(|_| Error::Config("PLEX_URL is required".into()))?;
        if plex_url.is_empty() {
            return Err(Error::Config("PLEX_URL is empty".into()));
        }
        // An explicit token wins. Otherwise read it from the Plex data
        // directory, so a container can mount that directory instead of
        // copying the token into its environment.
        let plex_token = match env::var("PLEX_TOKEN") {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => token_from_config(&plex_config_dir()?)?,
        };
        let language = Language::parse(&env::var("EDITION_LANG").unwrap_or_else(|_| "en".into()))?;
        let modules = parse_modules(
            &env::var("EDITION_MODULES").unwrap_or_else(|_| DEFAULT_MODULES.join(";")),
        )?;
        let skip_libraries = split_list(&env::var("EDITION_SKIP_LIBRARIES").unwrap_or_default())
            .into_iter()
            .filter(|name| !name.is_empty())
            .collect();
        let separator = env::var("EDITION_SEPARATOR").unwrap_or_else(|_| " · ".into());
        Ok(Self {
            plex_url,
            plex_token,
            modules,
            language,
            skip_libraries,
            separator,
            dry_run: false,
        })
    }
}

/// Where Plex keeps `Preferences.xml`. `PLEX_CONFIG_DIR` points straight at
/// the directory that holds the file; unset, the usual data directory is
/// assumed and the `Library/Application Support/...` tail is appended.
fn plex_config_dir() -> Result<PathBuf> {
    match env::var("PLEX_CONFIG_DIR") {
        Ok(value) if !value.trim().is_empty() => Ok(PathBuf::from(value.trim())),
        _ => Ok(PathBuf::from("/config").join(PREFERENCES_TAIL)),
    }
}

const PREFERENCES_TAIL: &str = "Library/Application Support/Plex Media Server";

fn token_from_config(dir: &Path) -> Result<String> {
    let path = dir.join("Preferences.xml");
    let text = fs::read_to_string(&path).map_err(|_| {
        Error::Config(format!(
            "PLEX_TOKEN is unset and {} cannot be read; set PLEX_TOKEN or PLEX_CONFIG_DIR",
            path.display()
        ))
    })?;
    plex_token_from_preferences(&text).ok_or_else(|| {
        Error::Config(format!(
            "{} has no PlexOnlineToken; the server may be unclaimed",
            path.display()
        ))
    })
}

/// `PlexOnlineToken` from a Plex `Preferences.xml`. The file is one element
/// with every setting as an attribute, so the value sits between quotes.
pub fn plex_token_from_preferences(text: &str) -> Option<String> {
    let key = "PlexOnlineToken=\"";
    let start = text.find(key)? + key.len();
    let rest = text.get(start..)?;
    let end = rest.find('"')?;
    let token = &rest[..end];
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(['\u{ff1b}', ';'])
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn parse_modules(raw: &str) -> Result<Vec<String>> {
    let mut modules = Vec::new();
    for name in split_list(raw) {
        let key = name.to_ascii_lowercase();
        let Some((_, id)) = MODULE_ALIASES.iter().find(|(alias, _)| *alias == key) else {
            return Err(Error::Config(format!("unknown module {name}")));
        };
        if !modules.iter().any(|existing| existing == id) {
            modules.push((*id).to_string());
        }
    }
    if modules.is_empty() {
        return Err(Error::Config("EDITION_MODULES is empty".into()));
    }
    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_chinese_aliases_and_fullwidth_separator() {
        let modules = parse_modules("片源版本；分辨率;动态范围").unwrap();
        assert_eq!(modules, ["source", "resolution", "dynamicrange"]);
    }

    #[test]
    fn rejects_unknown_modules() {
        assert!(parse_modules("not-a-module").is_err());
    }

    #[test]
    fn reads_the_token_attribute() {
        let text = r#"<?xml version="1.0" encoding="utf-8"?>
<Preferences MachineIdentifier="abc" PlexOnlineToken="tok-1" FriendlyName="Dev"/>"#;
        assert_eq!(plex_token_from_preferences(text).as_deref(), Some("tok-1"));
    }

    #[test]
    fn ignores_a_preferences_file_without_a_token() {
        let text = r#"<Preferences MachineIdentifier="abc" FriendlyName="Dev"/>"#;
        assert_eq!(plex_token_from_preferences(text), None);
        assert_eq!(
            plex_token_from_preferences(r#"<Preferences PlexOnlineToken=""/>"#),
            None
        );
    }
}
