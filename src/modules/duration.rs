use crate::config::Language;

pub fn duration(duration_ms: Option<u64>, language: Language) -> Option<String> {
    let minutes = (duration_ms? as f64 / 1000.0 / 60.0).round() as i64;
    Some(match language {
        Language::Zh => format!("{minutes} 分钟"),
        Language::En => format!("{minutes} MIN"),
    })
}
