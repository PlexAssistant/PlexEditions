pub fn resolution(raw: Option<&str>) -> Option<String> {
    let resolution = raw?.to_uppercase();
    if resolution.is_empty() {
        return None;
    }
    if resolution.chars().all(|ch| ch.is_ascii_digit()) {
        Some(format!("{resolution}P"))
    } else {
        Some(resolution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_gain_a_p() {
        assert_eq!(resolution(Some("1080")).as_deref(), Some("1080P"));
        assert_eq!(resolution(Some("4k")).as_deref(), Some("4K"));
    }
}
