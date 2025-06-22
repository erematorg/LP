pub mod fractals;
pub mod measures;

use bevy::prelude::*;

// Re-export main plugins following energy/forces pattern
pub use measures::MutualInformationPlugin;

/// Core trait for information processing systems
pub trait InformationProcessor: Send + Sync {
    /// Calculate information content for this system
    fn information_content(&self) -> f64;
    
    /// Get information type identifier
    fn information_type(&self) -> &'static str {
        "generic"
    }
    
    /// Calculate Shannon entropy for this system's state  
    /// Uses domain-independent entropy calculation
    fn entropy(&self) -> f64 {
        0.0 // Default implementation - override with actual state entropy
    }
}

/// Main plugin for all information-related systems
#[derive(Default)]
pub struct InformationPlugin;

impl Plugin for InformationPlugin {
    fn build(&self, app: &mut App) {
        // Add information theory systems
        app.add_plugins(MutualInformationPlugin)
           .insert_resource(InformationSystemsInitialized);
    }
}

/// Resource to indicate information systems are initialized
#[derive(Resource, Default)]
pub struct InformationSystemsInitialized;

pub mod prelude {
    // Main plugin export
    pub use crate::InformationPlugin;

    // Re-export from fractals module
    pub use crate::fractals::prelude::*;
    
    // Re-export from measures module
    pub use crate::measures::prelude::*;
}
