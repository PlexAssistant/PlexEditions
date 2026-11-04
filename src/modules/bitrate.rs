pub fn bitrate(kbps: Option<f64>) -> Option<String> {
    let kbps = kbps?;
    let mbps = kbps / 1000.0;
    if mbps < 1.0 {
        Some(format!("{} Kbps", kbps.round() as i64))
    } else {
        Some(format!("{:.1} Mbps", (mbps * 10.0).round() / 10.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_unit_at_one_megabit() {
        assert_eq!(bitrate(Some(800.4)).as_deref(), Some("800 Kbps"));
        assert_eq!(bitrate(Some(8450.0)).as_deref(), Some("8.5 Mbps"));
    }
}
