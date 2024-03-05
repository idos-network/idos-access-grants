const EVENT_JSON_PREFIX: &'static str = "EVENT_JSON";
const EVENT_JSON_SEPARATOR: &'static str = ":";
pub fn extract_event(s: &str) -> serde_json::Value {
    if let Some((EVENT_JSON_PREFIX, json_str)) = s.split_once(EVENT_JSON_SEPARATOR) {
        if let Ok(json_value) = json_str.parse::<serde_json::Value>() {
            return json_value;
        }
    }

    panic!(
        "Expected {:?} to start with {:?}, followed by {:?} and a valid JSON value.",
        s, EVENT_JSON_PREFIX, EVENT_JSON_SEPARATOR
    )
}
