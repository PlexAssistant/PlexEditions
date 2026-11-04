use crate::i18n::{self, Language};

pub fn country(names: &[String], language: Language) -> Option<String> {
    if names.is_empty() {
        return None;
    }
    let translated = names
        .iter()
        .map(|name| i18n::country(language, name))
        .collect::<Vec<_>>();
    Some(translated.join(" · "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_translated_countries() {
        let names = vec!["United States".into(), "Hong Kong".into()];
        assert_eq!(
            country(&names, Language::Zh).as_deref(),
            Some("美国 · 中国香港")
        );
        assert_eq!(
            country(&names, Language::En).as_deref(),
            Some("United States · Hong Kong SAR China")
        );
        assert_eq!(
            country(&names, Language::Fr).as_deref(),
            Some("États-Unis · Hong Kong")
        );
        assert_eq!(
            country(&names, Language::Ja).as_deref(),
            Some("アメリカ合衆国 · 香港")
        );
    }
}
