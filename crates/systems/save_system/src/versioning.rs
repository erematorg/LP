use serde_json::Value;

pub const SAVE_VERSION: &str = "0.1.0";

pub fn is_save_up_to_date(data: &Value) -> bool {
    let version = data
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    if version == *SAVE_VERSION {
        return true;
    }

    false
}

pub fn upgrade_save(mut data: Value) -> Value {
    let version = data
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");

    match version {
        "0.0.0" => {
            for (key, default_value) in get_default_fields() {
                if data.get(key).is_none() {
                    data[key] = default_value.clone();
                }
            }
        }
        "0.1.0" => {}
        _ => {
            for (key, default_value) in get_default_fields() {
                if data.get(key).is_none() {
                    data[key] = default_value.clone();
                }
            }
        }
    }

    data["version"] = SAVE_VERSION.into();
    data
}

fn get_default_fields() -> Vec<(&'static str, Value)> {
    vec![
        ("score", Value::from(42)),
        ("new_field", Value::from("default_value")),
    ]
}
