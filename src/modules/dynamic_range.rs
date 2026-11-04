use serde_json::Value;

pub fn dynamic_range(display_title: Option<&str>, dovi_profile: Option<&Value>) -> Option<String> {
    let display_title = display_title?;
    let hdr = display_title.contains("HDR");
    let dovi = display_title.contains("DoVi");
    let profile = dovi_profile.and_then(profile_text);
    let dynamic_range = match (hdr, dovi, profile) {
        (true, true, Some(profile)) => format!("HDR · DV P{profile}"),
        (true, _, _) => "HDR".into(),
        (false, true, Some(profile)) => format!("DV P{profile}"),
        _ => "SDR".into(),
    };
    Some(dynamic_range)
}

fn profile_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn combines_hdr_and_dovi() {
        assert_eq!(
            dynamic_range(Some("4K DoVi HDR10"), Some(&json!(7))).as_deref(),
            Some("HDR · DV P7")
        );
        assert_eq!(dynamic_range(Some("HDR10"), None).as_deref(), Some("HDR"));
        assert_eq!(dynamic_range(Some("SDR"), None).as_deref(), Some("SDR"));
    }
}
