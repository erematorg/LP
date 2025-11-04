pub mod traits;

use bevy::prelude::*;

/// Plugin for personality and behavioral trait systems
#[derive(Default)]
pub struct PersonalityPlugin;

impl Plugin for PersonalityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<traits::Personality>()
            .register_type::<traits::Altruistic>()
            .register_type::<traits::ContextAwareUtilities>()
            .register_type::<traits::PersonalityContextInputs>();
    }
}

/// Prelude for the personality module.
///
/// This includes personality traits and related components.
pub mod prelude {
    pub use crate::personality::PersonalityPlugin;
    pub use crate::personality::traits::{
        Altruistic, ContextAwareUtilities, Personality, PersonalityContextInputs,
    };
}
