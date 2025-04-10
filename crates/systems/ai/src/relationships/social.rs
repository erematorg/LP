use bevy::prelude::*;
use crate::core::utility::UtilityScore;
use std::collections::HashMap;


/// Entity identifier type (compatible with Bevy ECS)
pub type EntityId = Entity;

/// Normalized relationship value between entities
#[derive(Debug, Clone, Copy, Component)]
pub struct RelationshipStrength(f32);

impl RelationshipStrength {
   pub fn new(value: f32) -> Self {
       Self(value.clamp(0.0, 1.0))
   }
   
   pub fn value(&self) -> f32 {
       self.0
   }
   
   /// Convert relationship to utility score for decision-making
   pub fn to_utility(&self) -> UtilityScore {
       UtilityScore::new(self.0)
   }
   
   /// Modify relationship by amount (can be positive or negative)
   pub fn adjust(&mut self, amount: f32) {
       self.0 = (self.0 + amount).clamp(0.0, 1.0);
   }
}

/// Core ecological relationship types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum RelationshipType {
   Cooperation,  // Mutual benefit
   Competition,  // Resource rivals
   Predation,    // Hunter/prey dynamics
   Fear,         // Threat response
   Kinship,      // Family/genetic bonds
}

/// Single relationship between two entities
#[derive(Debug, Clone, Component)]
pub struct EntityRelationship {
   pub strength: RelationshipStrength,
   pub relationship_type: RelationshipType,
   pub last_interaction_tick: u64, // Game ticks since last interaction
}

/// Component that stores all relationships an entity maintains
#[derive(Debug, Default, Component)]
pub struct SocialNetwork {
   relationships: HashMap<EntityId, HashMap<RelationshipType, EntityRelationship>>,
}

impl SocialNetwork {
   /// Get all relationships of a specific ecological type
   pub fn get_by_type(&self, relationship_type: RelationshipType) -> Vec<(EntityId, &EntityRelationship)> {
       let mut result = Vec::new();
       
       for (&entity_id, relationships) in &self.relationships {
           if let Some(relationship) = relationships.get(&relationship_type) {
               result.push((entity_id, relationship));
           }
       }
       
       result
   }
}