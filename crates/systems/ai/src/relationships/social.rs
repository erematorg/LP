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

impl EntityRelationship {
    /// Update the last interaction tick to the current game tick
    pub fn update_interaction(&mut self, current_tick: u64) {
        self.last_interaction_tick = current_tick;
    }
}

/// Component that stores all relationships an entity maintains
#[derive(Debug, Default, Component)]
pub struct SocialNetwork {
   relationships: HashMap<EntityId, HashMap<RelationshipType, EntityRelationship>>,
}

impl SocialNetwork {
   /// Add a new relationship or update an existing one
   pub fn add_or_update_relationship(&mut self, target: EntityId, relationship_type: RelationshipType, strength: f32) {
       let relationship = EntityRelationship {
           strength: RelationshipStrength::new(strength),
           relationship_type,
           last_interaction_tick: 0,
       };

       self.relationships
           .entry(target)
           .or_default()
           .insert(relationship_type, relationship);
   }

   /// Query relationships with flexible filtering
   pub fn query_relationships(
       &self, 
       relationship_type: Option<RelationshipType>, 
       min_strength: Option<f32>
   ) -> Vec<(EntityId, &EntityRelationship)> {
       let mut result = Vec::new();
       
       for (&entity_id, relationships) in &self.relationships {
           for (current_type, relationship) in relationships {
               let type_match = relationship_type.map_or(true, |rt| *current_type == rt);
               let strength_match = min_strength.map_or(true, |ms| relationship.strength.value() >= ms);
               
               if type_match && strength_match {
                   result.push((entity_id, relationship));
               }
           }
       }
       
       result
   }
}