use serde_json::Value;

pub const SAVE_VERSION: &str = "0.1.0";

pub fn is_save_up_to_date(data: &Value) -> bool {
    data.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        == SAVE_VERSION
}

/// Upgrade a save file to the current version by stepping through each migration in sequence.
/// Add new steps when SAVE_VERSION bumps. Never remove a step — old saves must stay upgradeable.
pub fn upgrade_save(mut data: Value) -> Value {
    // Each entry: (from_version, to_version, migration_fn)
    const STEPS: &[(&str, &str, fn(Value) -> Value)] = &[
        ("0.0.0", "0.1.0", migrate_0_0_0_to_0_1_0),
        // ("0.1.0", "0.2.0", migrate_0_1_0_to_0_2_0),
    ];

    for &(from_ver, to_ver, migrate_fn) in STEPS {
        let current = data
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        if current == SAVE_VERSION {
            break;
        }

        if current == from_ver {
            data = migrate_fn(data);
            data["version"] = Value::from(to_ver);
        }
    }

    data
}

fn migrate_0_0_0_to_0_1_0(mut data: Value) -> Value {
    // Fill any LP simulation fields that must exist in 0.1.0 saves.
    // Extend as LP gains persistent state (sim_seed, active_bodies, etc.).
    let defaults: &[(&str, Value)] = &[
        ("version", Value::from(SAVE_VERSION)),
        ("sim_time", Value::from(0.0_f64)),
    ];
    for (key, default_value) in defaults {
        if data.get(key).is_none() {
            data[key] = default_value.clone();
        }
    }
    data
}
