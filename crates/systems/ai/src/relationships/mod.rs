pub mod social;

/// Prelude for the relationships module.
///
/// This includes social relationships and network components.
pub mod prelude {
    pub use crate::relationships::social::{
        get_relationship_strength, EntityRelationship, RelationshipStrength, RelationshipType,
        SocialNetwork,
    };
}
