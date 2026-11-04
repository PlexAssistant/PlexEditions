pub fn video_codec(raw: Option<&str>) -> Option<String> {
    let codec = raw?.to_uppercase();
    Some(match codec.as_str() {
        "MPEG2VIDEO" => "MPEG2".into(),
        "MPEG1VIDEO" => "MPEG1".into(),
        _ => codec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaps_the_two_known_codecs() {
        assert_eq!(video_codec(Some("hevc")).as_deref(), Some("HEVC"));
        assert_eq!(video_codec(Some("mpeg1video")).as_deref(), Some("MPEG1"));
    }
}
