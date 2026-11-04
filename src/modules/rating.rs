use serde_json::Value;

pub fn rating(raw: Option<&Value>) -> Option<String> {
    match raw? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}
