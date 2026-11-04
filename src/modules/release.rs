use super::search;
use crate::i18n::{self, Language};

pub fn release(
    file_name: &str,
    embedded_title: Option<&str>,
    language: Language,
) -> Option<String> {
    let from_name = labels(file_name, language);
    let matched = if from_name.is_empty() {
        embedded_title
            .map(|title| labels(&title.to_uppercase(), language))
            .unwrap_or_default()
    } else {
        from_name
    };
    if matched.is_empty() {
        None
    } else {
        Some(matched.join(" · "))
    }
}

fn labels(text: &str, language: Language) -> Vec<String> {
    i18n::releases()
        .into_iter()
        .filter(|entry| search(entry.pattern, text))
        .map(|entry| i18n::text(language, entry.labels))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_every_hit() {
        assert_eq!(
            release("Movie.IMAX.3D.mkv", None, Language::Zh).as_deref(),
            Some("3D 版 · IMAX 版")
        );
        assert_eq!(
            release("Movie.IMAX.mkv", None, Language::Es).as_deref(),
            Some("Edición IMAX")
        );
    }

    #[test]
    fn bluray_token_is_a_release() {
        assert_eq!(
            release("Movie.BD.mkv", None, Language::En).as_deref(),
            Some("Blu-ray Edition")
        );
        assert_eq!(
            release("Movie.BD.mkv", None, Language::Fr).as_deref(),
            Some("Édition Blu-ray")
        );
    }

    #[test]
    fn matches_regardless_of_case() {
        assert_eq!(
            release("Movie.imax.mkv", None, Language::En).as_deref(),
            Some("IMAX Edition")
        );
        assert_eq!(
            release("movie special edition 1080p.mkv", None, Language::Zh).as_deref(),
            Some("特别版")
        );
    }
}
