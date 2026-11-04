pub fn frame_rate(raw: Option<&str>) -> Option<String> {
    let rate = raw?.to_uppercase();
    Some(match rate.as_str() {
        "NTSC" => "30P".into(),
        "PAL" => "25P".into(),
        "FILM" => "24P".into(),
        other => other.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_named_rates() {
        assert_eq!(frame_rate(Some("FILM")).as_deref(), Some("24P"));
        assert_eq!(frame_rate(Some("24p")).as_deref(), Some("24P"));
    }
}
