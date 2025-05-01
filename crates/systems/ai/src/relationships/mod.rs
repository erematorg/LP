pub mod social;

/// Prelude for the relationships module.
///
/// This includes social relationships and network components.
pub mod prelude {
    pub use crate::relationships::social::{SocialNetwork, RelationshipType, RelationshipStrength, 
                                         EntityRelationship, get_relationship_strength};
}