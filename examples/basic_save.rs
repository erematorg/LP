use save_system::save_system::{save, load};
use save_system::versioning::SAVE_VERSION;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    version: String,
    score: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION.to_string(), // Ensures the correct version is set
            score: 42,
        }
    }
}

fn main() {
    let path = "save.json";

    let mut data = match load::<GameData>(path) {
        Ok(loaded_data) => {
            println!("Loaded existing data: {:?}", loaded_data);
            loaded_data
        }
        Err(e) => {
            eprintln!("Failed to load existing save: {}. Creating a new one.", e);
            GameData { version: SAVE_VERSION.to_string(), score: 42 }
        }
    };

    // Modify the score every run to show persistence
    data.score += 1;

    if let Err(e) = save(&data, path) {
        eprintln!("Save failed: {}", e);
    } else {
        println!("Data saved successfully!");
    }
}
