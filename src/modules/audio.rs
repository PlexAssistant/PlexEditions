use crate::config::Language;

pub fn audio_codec(display_title: Option<&str>, language: Language) -> Option<String> {
    let display_title = display_title?;
    let start = display_title.rfind('(');
    let end = display_title.rfind(')');
    let mut info = if let (Some(start), Some(end)) = (start, end) {
        if end > start {
            display_title[start + 1..end].to_string()
        } else {
            display_title.to_string()
        }
    } else {
        display_title.to_string()
    };
    if language == Language::Zh {
        info = info.replace("Mono", "单声道").replace("Stereo", "立体声");
    }
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_last_parenthetical() {
        assert_eq!(
            audio_codec(Some("English (TRUEHD 7.1)"), Language::En).as_deref(),
            Some("TRUEHD 7.1")
        );
        assert_eq!(
            audio_codec(Some("Commentary (Stereo)"), Language::Zh).as_deref(),
            Some("立体声")
        );
    }
}
