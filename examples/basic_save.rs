use serde::{Deserialize, Serialize};
use systems::save_system::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    version: String,
    score: u32,
    new_field: String,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION.to_string(),
            score: 42,
            new_field: "default_value".to_string(),
        }
    }
}

fn main() {
    let path = "save.json";
    let mut data = match load::<GameData>(path) {
        Ok(data) => data,
        Err(_) => GameData::default(),
    };

    data.score += 1;
    println!("Score: {}", data.score);

    if save(&data, path).is_err() {
        eprintln!("Save failed");
    }

    let mut tracker = GameTracker::default();
    tracker.set_total_energy(1500.0);
    tracker.set_entity_count(25);
    tracker.set_environment("temperature".to_string(), 22.5);
    tracker.log_event(
        "spawn".to_string(),
        Some("creature_01".to_string()),
        "spawned".to_string(),
        120.5,
    );
    tracker.log_event(
        "energy_transfer".to_string(),
        Some("creature_01".to_string()),
        "consumed 50 energy".to_string(),
        125.0,
    );

    let game_save = GameSaveData {
        version: data.version.clone(),
        timestamp: 1000.0,
        game_time: 130.0,
        metadata: Default::default(),
        game_state: tracker.state.clone(),
        events: tracker.events.clone(),
        entities: std::collections::HashMap::new(),
    };

    match save(&game_save, path) {
        Ok(_) => {
            println!("Energy: {}", game_save.game_state.total_energy);
            println!("Entities: {}", game_save.game_state.entity_count);
            println!("Events: {}", game_save.events.len());
        }
        Err(e) => eprintln!("Save failed: {}", e),
    }

    if let Ok(loaded) = load::<GameSaveData>(path) {
        println!("Game time: {}", loaded.game_time);
        for event in &loaded.events {
            println!("{}: {}", event.event_type, event.data);
        }
    }
}
