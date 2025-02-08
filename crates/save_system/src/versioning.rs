use serde_json::Value;

pub const SAVE_VERSION: &str = "1.0.0";
pub const VERSION_HISTORY: &[&str] = &[
    "1.0.0", // First version
];

/// Ensures the version history is ordered correctly (Godot-inspired check)
pub fn validate_versioning() -> bool {
    for i in 1..VERSION_HISTORY.len() {
        if VERSION_HISTORY[i] < VERSION_HISTORY[i - 1] {
            eprintln!("[Error] Version history is not in order.");
            return false;
        }
    }
    true
}

/// Checks if a save file is up to date
pub fn is_save_up_to_date(data: &Value) -> bool {
    if let Some(version) = data.get("version").and_then(|v| v.as_str()) {
        return version == SAVE_VERSION;
    }

    eprintln!("[Warning] Save file does not have a version number.");
    false
}
