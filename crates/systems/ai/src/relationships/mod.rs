pub mod social;

use bevy::prelude::*;

/// Plugin for social relationship systems
#[derive(Default)]
pub struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<social::SocialConfig>()
            .register_type::<social::SocialConfig>()
            .register_type::<social::SocialNetwork>()
            .register_type::<social::RelationshipStrength>()
            .register_type::<social::RelationshipType>()
            .register_type::<social::EntityRelationship>()
            .register_type::<social::SocialRelation>();
    }
}

/// Prelude for the relationships module.
///
/// This includes social relationships and network components.
pub mod prelude {
    pub use crate::relationships::SocialPlugin;
    pub use crate::relationships::social::{
        EntityRelationship, RelationshipStrength, RelationshipType, SocialConfig, SocialNetwork,
        SocialRelation, get_relationship_strength,
    };
}
