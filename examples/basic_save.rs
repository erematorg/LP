use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use systems::save_system::prelude::*;

// Example custom payload saved alongside the LP simulation state.
#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    version: String,
    score: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION.to_string(),
            score: 42,
        }
    }
}

fn main() {
    let settings = SaveSettings::default();
    let path = "save.json";

    // ── Basic save/load ───────────────────────────────────────────────────────
    let mut data = load::<GameData>(&settings, path).unwrap_or_default();
    data.score += 1;
    println!("Score: {}", data.score);
    if save(&data, &settings, path).is_err() {
        eprintln!("Save failed");
    }

    // ── PersistentId: stable cross-session entity identity ────────────────────
    // IDs are seeded from system time + PID — collision-free across sessions
    // without needing a UUID library.
    let mut id_counter = PersistentIdCounter::default();
    let star_id = id_counter.generate();
    let planet_id = id_counter.generate();
    println!("Star id={} Planet id={}", star_id.0, planet_id.0);

    // ── GameTracker: simulation state + event log ─────────────────────────────
    let mut tracker = GameTracker::default();
    tracker.set_total_energy(1500.0);
    tracker.set_entity_count(2);
    tracker.set_environment("temperature".to_string(), 5778.0);
    tracker.log_event(
        "spawn".to_string(),
        Some(format!("{}", star_id.0)),
        "star spawned".to_string(),
        0.0,
    );
    tracker.log_event(
        "orbit_start".to_string(),
        Some(format!("{}", planet_id.0)),
        "planet entered stable orbit".to_string(),
        1.2,
    );

    // ── Persist entity components by PersistentId ─────────────────────────────
    let mut entities: HashMap<String, HashMap<String, Value>> = HashMap::new();
    entities.insert(
        star_id.0.to_string(),
        HashMap::from([
            ("mass".to_string(), json!(2e30_f64)),
            ("radius".to_string(), json!(696_000_f64)),
        ]),
    );
    entities.insert(
        planet_id.0.to_string(),
        HashMap::from([
            ("mass".to_string(), json!(6e24_f64)),
            ("orbit_radius".to_string(), json!(1.5e11_f64)),
        ]),
    );

    let game_save = GameSaveData {
        version: SAVE_VERSION.to_string(),
        timestamp: 1000.0,
        game_time: 1.2,
        metadata: Default::default(),
        game_state: tracker.state.clone(),
        events: tracker.events.iter().cloned().collect(), // VecDeque → Vec for serialisation
        entities,
    };

    match save(&game_save, &settings, path) {
        Ok(location) => {
            println!("Saved to {}", location.display());
            println!(
                "  energy={} entities={} events={}",
                game_save.game_state.total_energy,
                game_save.game_state.entity_count,
                game_save.events.len(),
            );
        }
        Err(e) => eprintln!("Save failed: {}", e),
    }

    // ── Reload and query by PersistentId ─────────────────────────────────────
    if let Ok(loaded) = load::<GameSaveData>(&settings, path) {
        println!("Loaded game_time={}", loaded.game_time);

        // Look up a specific entity's saved components
        if let Some(components) = get_saved_entity_components(&loaded, star_id) {
            println!("Star mass={}", components.get("mass").unwrap());
        }

        for event in &loaded.events {
            println!("  event={} data={}", event.event_type, event.data);
        }
    }

    // ── Version migration smoke-test ──────────────────────────────────────────
    let old_save = serde_json::json!({ "version": "0.0.0", "score": 99 });
    let migrated = upgrade_save(old_save);
    println!(
        "Migrated version: {} (up-to-date: {})",
        migrated["version"],
        is_save_up_to_date(&migrated)
    );

    let _ = delete_save_file(&settings, path);
}
