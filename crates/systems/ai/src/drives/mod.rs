pub mod needs;

use bevy::prelude::*;

/// Plugin for biological drive systems
#[derive(Default)]
pub struct DrivesPlugin;

impl Plugin for DrivesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<needs::Need>()
            .register_type::<needs::NeedType>();
        // Simple plugin - just makes drives available
        // Systems will be added later when we have proper integration
    }
}

/// Prelude for the drives module.
///
/// This includes core need types and drive components.
pub mod prelude {
    pub use crate::drives::needs::{get_most_urgent_need, update_needs, Need, NeedType};
    pub use crate::drives::DrivesPlugin;
}
