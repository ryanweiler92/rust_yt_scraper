pub fn string_to_bool(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" | "on" => Some(true),
        "false" | "no" | "n" | "0" | "off" => Some(false),
        _ => None,
    }
}