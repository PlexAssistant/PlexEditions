use super::search;
use crate::i18n::{self, Language};

pub fn cut(file_name: &str, embedded_title: Option<&str>, language: Language) -> Option<String> {
    label_of(file_name, language)
        .or_else(|| embedded_title.and_then(|title| label_of(&title.to_uppercase(), language)))
}

fn label_of(text: &str, language: Language) -> Option<String> {
    i18n::cuts()
        .into_iter()
        .find_map(|entry| search(entry.pattern, text).then(|| i18n::text(language, entry.labels)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_picks_the_label() {
        assert_eq!(
            cut("Movie.Directors.Cut.mkv", None, Language::En).as_deref(),
            None
        );
        assert_eq!(
            cut("Movie.Director's Cut.mkv", None, Language::Zh).as_deref(),
            Some("导演剪辑版")
        );
        assert_eq!(
            cut("Movie.DC.mkv", None, Language::Fr).as_deref(),
            Some("Version du réalisateur")
        );
        assert_eq!(
            cut("Movie.DC.mkv", None, Language::Ru).as_deref(),
            Some("Режиссёрская версия")
        );
        assert_eq!(
            cut("Movie.DC.mkv", None, Language::Es).as_deref(),
            Some("Versión del director")
        );
        assert_eq!(
            cut("Movie.DC.mkv", None, Language::Ar).as_deref(),
            Some("نسخة المخرج")
        );
    }

    #[test]
    fn matches_regardless_of_case() {
        // Release names shout the tokens; the word lists are mixed-case.
        assert_eq!(
            cut("Movie.EXTENDED.1080p.mkv", None, Language::Zh).as_deref(),
            Some("加长版")
        );
        assert_eq!(
            cut("Movie.theatrical.mkv", None, Language::En).as_deref(),
            Some("Theatrical Cut")
        );
    }
}
