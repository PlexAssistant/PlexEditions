use super::search;

/// Ordered first-hit rules. TC and TS keep their original place, including
/// the extension exception, so a later rule cannot steal the match.
const RULES: &[(&str, &str, bool)] = &[
    (r"\b(REMUX|BDREMUX|BD-REMUX)\b", "REMUX", false),
    (r"\b(BLURAY|BD|BLU-RAY|BD1080P)\b|蓝光", "BD", false),
    (r"\bBDRIP\b", "BDRIP", false),
    (r"\bWEB-DL|WEBDL\b", "WEB-DL", false),
    (r"\bVODRIP\b", "VODRIP", false),
    (r"\bWEBRIP\b", "WEBRIP", false),
    (r"\bHDRIP\b", "HDRIP", false),
    (r"\bHR-HDTV|HRHDTV\b", "HR-HDTV", false),
    (r"\bHDTV\b", "HDTV", false),
    (r"\bPDTV\b", "PDTV", false),
    (r"\bDVD\b", "DVD", false),
    (r"\bDVDRIP\b", "DVDRIP", false),
    (r"\bDVDSCR\b", "DVDSCR", false),
    (r"\bR5\b", "R5", false),
    (r"\bLDRIP\b", "LDRIP", false),
    (r"\bPPVRIP\b", "PPVRIP", false),
    (r"\bSDTV\b", "SDTV", false),
    (r"\bTVRIP\b", "TVRIP", false),
    (r"\bVHSRIP\b", "VHSRIP", false),
    (r"\bHDTC|HD-TC\b", "HDTC", false),
    (r"\bTC\b", "TC", true),
    (r"\bHDCAM|HD-CAM\b", "HDCAM", false),
    (r"\bHQCAM|HQ-CAM\b", "HQCAM", false),
    (r"\bTS\b", "TS", true),
    (r"\bCAM\b", "CAM", false),
];

pub fn source(file_name: &str, embedded_title: Option<&str>) -> Option<String> {
    match_source(&file_name.to_uppercase())
        .or_else(|| embedded_title.and_then(|title| match_source(&title.to_uppercase())))
}

fn match_source(title: &str) -> Option<String> {
    for (pattern, label, skip_extension) in RULES {
        if search(pattern, title) && !(*skip_extension && title.ends_with(&format!(".{label}"))) {
            return Some((*label).to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_hit_wins_and_chinese_bluray_is_bd() {
        assert_eq!(
            source("Movie.REMUX.BluRay.mkv", None).as_deref(),
            Some("REMUX")
        );
        assert_eq!(source("Movie.蓝光.mkv", None).as_deref(), Some("BD"));
        assert_eq!(source("Movie.WEBDL.mkv", None).as_deref(), Some("WEB-DL"));
    }

    #[test]
    fn extension_is_not_a_source() {
        assert_eq!(source("Movie.TS", None), None);
        assert_eq!(source("Movie.TC", None), None);
        assert_eq!(source("Movie.TC.mkv", None).as_deref(), Some("TC"));
        assert_eq!(source("Movie.HDCAM.mkv", None).as_deref(), Some("HDCAM"));
    }

    #[test]
    fn falls_back_to_embedded_title() {
        assert_eq!(
            source("Movie.strm", Some("WEB-DL")).as_deref(),
            Some("WEB-DL")
        );
    }
}
