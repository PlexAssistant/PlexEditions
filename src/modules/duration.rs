use crate::i18n::{self, Language};

pub fn duration(duration_ms: Option<u64>, language: Language) -> Option<String> {
    let minutes = (duration_ms? as f64 / 1000.0 / 60.0).round() as i64;
    Some(format!("{minutes} {}", i18n::term(language, "minutes")))
}
