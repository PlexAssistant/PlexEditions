use super::search;
use crate::config::Language;

const RELEASES: &[(&str, &str, &str)] = &[
    (r"\b(Special Edition)\b|特别版", "Special Edition", "特别版"),
    (
        r"\b(Restored Edition)\b|数字修复版|数字修复",
        "Restored Edition",
        "数字修复版",
    ),
    (r"\b(3D Edition|3D)\b|3D版", "3D Edition", "3D 版"),
    (r"\b(IMAX Edition|IMAX)\b|IMAX版", "IMAX Edition", "IMAX 版"),
    (
        r"\b(Collector's Edition)\b|收藏版",
        "Collector's Edition",
        "收藏版",
    ),
    (
        r"\b(Anniversary Edition)\b|周年纪念版",
        "Anniversary Edition",
        "周年纪念版",
    ),
    (
        r"\b(Ultimate Edition)\b|终极版",
        "Ultimate Edition",
        "终极版",
    ),
    (
        r"\b(Blu-ray Edition|Blu-ray|Bluray|BD)\b|蓝光版|蓝光",
        "Blu-ray Edition",
        "蓝光版",
    ),
    (r"\b(DVD Edition|DVD)\b|DVD版", "DVD Edition", "DVD 版"),
    (
        r"\b(Limited Edition|Limited)\b|限量版",
        "Limited Edition",
        "限量版",
    ),
    (
        r"\b(Commemorative Edition)\b|纪念版",
        "Commemorative Edition",
        "纪念版",
    ),
    (r"\b(Deluxe Edition)\b|豪华版", "Deluxe Edition", "豪华版"),
    (
        r"\b(Director's Signature Edition|Director's Signature)\b|导演签名版",
        "Director's Signature Edition",
        "导演签名版",
    ),
    (
        r"\b(Criterion Collection|CC)\b|标准收藏版|标准收藏",
        "Criterion Collection",
        "标准收藏版",
    ),
];

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
    RELEASES
        .iter()
        .filter(|(pattern, _, _)| search(pattern, text))
        .map(|(_, english, chinese)| {
            if language == Language::Zh {
                (*chinese).to_string()
            } else {
                (*english).to_string()
            }
        })
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
    }

    #[test]
    fn bluray_token_is_a_release() {
        assert_eq!(
            release("Movie.BD.mkv", None, Language::En).as_deref(),
            Some("Blu-ray Edition")
        );
    }
}
