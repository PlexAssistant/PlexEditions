use super::search;
use crate::config::Language;

const CUTS: &[(&str, &str, &str)] = &[
    (
        r"\b(Theatrical Cut|Theatrical)\b|院线版|院线|公映版|公映",
        "Theatrical Cut",
        "院线版",
    ),
    (
        r"\b(Director's Cut|DC)\b|导演剪辑版|导演剪辑",
        "Director's Cut",
        "导演剪辑版",
    ),
    (
        r"\b(Producer's Cut)\b|制片人剪辑版|制片人剪辑",
        "Producer's Cut",
        "制片人剪辑版",
    ),
    (
        r"\b(Extended Cut|Extended)\b|加长版|加长",
        "Extended Cut",
        "加长版",
    ),
    (
        r"\b(Unrated Cut|Unrated)\b|未分级版|未分级",
        "Unrated Cut",
        "未分级版",
    ),
    (
        r"\b(The Final Cut|Final Cut)\b|最终剪辑版|最终剪辑",
        "Final Cut",
        "最终剪辑版",
    ),
    (r"\b(Television Cut)\b|电视版", "Television Cut", "电视版"),
    (
        r"\b(International Cut)\b|国际版",
        "International Cut",
        "国际版",
    ),
    (
        r"\b(Home Video Cut)\b|家庭录像版",
        "Home Video Cut",
        "家庭录像版",
    ),
    (r"\b(Rough Cut)\b|初剪版", "Rough Cut", "初剪版"),
    (
        r"\b(Workprint Cut|Workprint)\b|工作版",
        "Workprint Cut",
        "工作版",
    ),
    (
        r"\b(Fan Edit)\b|粉丝剪辑版|粉丝剪辑",
        "Fan Edit",
        "粉丝剪辑版",
    ),
    (r"\b(Redux)\b", "Redux", ""),
];

pub fn cut(file_name: &str, embedded_title: Option<&str>, language: Language) -> Option<String> {
    label_of(file_name, language)
        .or_else(|| embedded_title.and_then(|title| label_of(&title.to_uppercase(), language)))
}

fn label_of(text: &str, language: Language) -> Option<String> {
    CUTS.iter().find_map(|(pattern, english, chinese)| {
        if search(pattern, text) {
            let label = if language == Language::Zh {
                *chinese
            } else {
                *english
            };
            Some(label.to_string())
        } else {
            None
        }
    })
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
            cut("Movie.DC.mkv", None, Language::En).as_deref(),
            Some("Director's Cut")
        );
    }

    #[test]
    fn redux_chinese_label_is_empty() {
        assert_eq!(
            cut("Movie.Redux.mkv", None, Language::Zh).as_deref(),
            Some("")
        );
    }
}
