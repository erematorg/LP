use save_system::save_system::{save, load};
use save_system::versioning::SAVE_VERSION;
use serde::{Serialize, Deserialize};

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
        Ok(loaded_data) => {
            println!("Loaded existing data: {:?}", loaded_data);
            loaded_data
        }
        Err(e) => {
            eprintln!("Failed to load existing save: {}. Creating a new one.", e);
            GameData::default()
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
