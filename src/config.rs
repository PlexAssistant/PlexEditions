use std::collections::HashSet;
use std::env;

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
        let plex_token = env::var("PLEX_TOKEN")
            .map(|value| value.trim().to_string())
            .map_err(|_| Error::Config("PLEX_TOKEN is required".into()))?;
        if plex_token.is_empty() {
            return Err(Error::Config("PLEX_TOKEN is empty".into()));
        }
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
}
