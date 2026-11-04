use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::error::{Error, Result};

const CATALOG_JSON: &str = include_str!("../locales/edition.json");

/// Languages written into an edition. Logs stay in English.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Zh,
    ZhTw,
    Ja,
    Ko,
    Ar,
    Fr,
    Ru,
    Es,
}

impl Language {
    pub fn parse(raw: &str) -> Result<Self> {
        let raw = raw.trim().to_ascii_lowercase().replace('_', "-");
        match raw.as_str() {
            "en" => Ok(Self::En),
            "zh" | "zh-cn" | "zh-hans" => Ok(Self::Zh),
            "zh-tw" | "zh-hant" => Ok(Self::ZhTw),
            "ja" => Ok(Self::Ja),
            "ko" => Ok(Self::Ko),
            "ar" => Ok(Self::Ar),
            "fr" => Ok(Self::Fr),
            "ru" => Ok(Self::Ru),
            "es" => Ok(Self::Es),
            other => Err(Error::Config(format!(
                "EDITION_LANG must be en, zh, zh-TW, ja, ko, ar, fr, ru, or es, got {other}"
            ))),
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh-Hans",
            Self::ZhTw => "zh-Hant",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Ar => "ar",
            Self::Fr => "fr",
            Self::Ru => "ru",
            Self::Es => "es",
        }
    }
}

#[derive(Debug, Deserialize)]
struct Catalog {
    terms: HashMap<String, HashMap<String, String>>,
    cuts: Vec<LabeledPattern>,
    releases: Vec<LabeledPattern>,
    countries: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct LabeledPattern {
    pattern: String,
    labels: HashMap<String, String>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(CATALOG_JSON).expect("edition catalog"))
}

pub fn term(language: Language, name: &str) -> &'static str {
    catalog()
        .terms
        .get(name)
        .and_then(|labels| labels.get(language.key()).or_else(|| labels.get("en")))
        .map(String::as_str)
        .unwrap_or("")
}

pub fn text(language: Language, labels: &HashMap<String, String>) -> String {
    labels
        .get(language.key())
        .or_else(|| labels.get("en"))
        .cloned()
        .unwrap_or_default()
}

pub struct Pattern {
    pub pattern: &'static str,
    pub labels: &'static HashMap<String, String>,
}

pub fn cuts() -> Vec<Pattern> {
    patterns(&catalog().cuts)
}

pub fn releases() -> Vec<Pattern> {
    patterns(&catalog().releases)
}

fn patterns(entries: &'static [LabeledPattern]) -> Vec<Pattern> {
    entries
        .iter()
        .map(|entry| Pattern {
            pattern: entry.pattern.as_str(),
            labels: &entry.labels,
        })
        .collect()
}

pub fn country(language: Language, name: &str) -> String {
    catalog()
        .countries
        .get(name)
        .and_then(|labels| labels.get(language.key()).or_else(|| labels.get("en")))
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_configured_language() {
        assert_eq!(Language::parse("zh-TW").unwrap(), Language::ZhTw);
        assert_eq!(Language::parse("zh-Hans").unwrap(), Language::Zh);
        assert_eq!(Language::parse("ar").unwrap(), Language::Ar);
        assert_eq!(Language::parse("fr").unwrap(), Language::Fr);
        assert_eq!(Language::parse("ru").unwrap(), Language::Ru);
        assert_eq!(Language::parse("es").unwrap(), Language::Es);
        assert!(Language::parse("de").is_err());
    }

    #[test]
    fn catalog_covers_nine_languages() {
        let catalog = catalog();
        for labels in catalog.terms.values() {
            assert_eq!(labels.len(), 9, "{labels:?}");
        }
        assert_eq!(catalog.cuts.len(), 13);
        assert_eq!(catalog.releases.len(), 14);
        assert!(catalog.countries.contains_key("France"));
        assert_eq!(country(Language::Fr, "France"), "France");
        assert_eq!(country(Language::Ar, "France"), "فرنسا");
        assert_eq!(term(Language::Es, "minutes"), "MIN");
    }

    #[test]
    fn release_labels_stay_distinct_per_language() {
        // Two releases on one file both render; identical labels would
        // write the same word twice.
        let catalog = catalog();
        for lang in [
            Language::En,
            Language::Zh,
            Language::ZhTw,
            Language::Ja,
            Language::Ko,
            Language::Ar,
            Language::Fr,
            Language::Ru,
            Language::Es,
        ] {
            let mut seen = Vec::new();
            for entry in &catalog.releases {
                let label = text(lang, &entry.labels);
                assert!(!label.is_empty(), "empty release label for {lang:?}");
                assert!(
                    !seen.contains(&label),
                    "duplicate release label {label:?} in {lang:?}"
                );
                seen.push(label);
            }
        }
    }

    #[test]
    fn accepts_underscore_language_codes() {
        assert_eq!(Language::parse("zh_TW").unwrap(), Language::ZhTw);
        assert_eq!(Language::parse("ZH-HANT").unwrap(), Language::ZhTw);
    }

    #[test]
    fn every_country_has_real_translations() {
        // A row that equals its English label in all seven translated
        // columns means the catalog build silently fell back.
        let translated = [
            Language::ZhTw,
            Language::Ja,
            Language::Ko,
            Language::Ar,
            Language::Fr,
            Language::Ru,
            Language::Es,
        ];
        let all = [
            Language::En,
            Language::Zh,
            Language::ZhTw,
            Language::Ja,
            Language::Ko,
            Language::Ar,
            Language::Fr,
            Language::Ru,
            Language::Es,
        ];
        for (name, labels) in &catalog().countries {
            let english = labels.get("en").cloned().unwrap_or_default();
            let fallback = translated
                .iter()
                .all(|lang| labels.get(lang.key()) == Some(&english));
            assert!(!fallback, "{name} fell back to English");
            for lang in all {
                assert!(
                    labels
                        .get(lang.key())
                        .is_some_and(|label| !label.is_empty()),
                    "{name} missing {lang:?}"
                );
            }
        }
    }
}
