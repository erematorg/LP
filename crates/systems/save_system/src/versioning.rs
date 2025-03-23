use serde_json::Value;

pub const SAVE_VERSION: &str = "0.1.0"; // Fix to match Cargo.toml

/// Checks if a save file is up to date
pub fn is_save_up_to_date(data: &Value) -> bool {
    let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
    if version == SAVE_VERSION.to_string() {
        return true;
    }

    eprintln!("[Warning] Save file is outdated! Detected version: {}. Expected: {}.", version, SAVE_VERSION);
    false
}

/// Upgrades old save data to match the latest format dynamically
pub fn upgrade_save(mut data: Value) -> Value {
    let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

    // Always upgrade to the latest version
    eprintln!("[Info] Upgrading save from {} to {}...", version, SAVE_VERSION);

    // Ensure missing fields are initialized dynamically
    for (key, default_value) in get_default_fields() {
        if !data.get(key).is_some() {
            eprintln!("[Info] Adding missing field: {}", key);
            data[key] = default_value.clone(); // Ensure the value is actually added
        }
    }

    // Rename any fields if needed
    rename_property(&mut data, "player_info", "player_data");

    // Set version to latest
    data["version"] = SAVE_VERSION.into();

    data
}

/// Returns a map of all expected fields with their default values
fn get_default_fields() -> Vec<(&'static str, Value)> {
    vec![
        ("score", Value::from(42)),  // Default score
        ("new_field", Value::from("default_value")), // Ensure this field is correctly added
    ]
}

/// Renames a property in the JSON data
fn rename_property(data: &mut Value, old_key: &str, new_key: &str) {
    if let Some(value) = data.get(old_key).cloned() {
        data[new_key] = value;
        data.as_object_mut().unwrap().remove(old_key);
        eprintln!("[Info] Renamed property: {} → {}", old_key, new_key);
    }
}
