use bevy::prelude::*;
use crate::prelude::*;
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
   pub fn add_or_update_relationship(&mut self, target: EntityId, relationship_type: RelationshipType, strength: f32, current_tick: u64) {
       let mut relationship = EntityRelationship {
           strength: RelationshipStrength::new(strength),
           relationship_type,
           last_interaction_tick: current_tick,
       };

       // If relationship already exists, update based on existing interaction history
       if let Some(existing_relationship) = self.relationships
           .entry(target)
           .or_default()
           .get_mut(&relationship_type) {
           
           // Modify strength based on interaction frequency
           relationship.strength.adjust(
               (current_tick - existing_relationship.last_interaction_tick) as f32 / 1000.0
           );
       }

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
   
   /// Adjust relationship based on personality traits
   pub fn adjust_relationship_with_personality(
       &mut self, 
       target: EntityId, 
       relationship_type: RelationshipType, 
       personality: &Personality, 
       current_tick: u64
   ) {
       // Base strength from existing relationship or default
       let base_strength = self.relationships
           .get(&target)
           .and_then(|r| r.get(&relationship_type))
           .map_or(0.5, |rel| rel.strength.value());
       
       // Apply personality modifiers
       let modified_strength = match relationship_type {
           RelationshipType::Cooperation => {
               // More dominant personalities are less cooperative
               base_strength * (1.0 - personality.dominance * 0.3)
           },
           RelationshipType::Competition => {
               // More aggressive personalities compete more strongly
               base_strength * (1.0 + personality.aggression * 0.5)
           },
           RelationshipType::Predation => {
               // More aggressive and brave personalities have stronger predation
               base_strength * (1.0 + personality.aggression * 0.6 + personality.bravery * 0.4)
           },
           RelationshipType::Fear => {
               // Less brave personalities feel more fear
               base_strength * (1.0 + (1.0 - personality.bravery) * 0.7)
           },
           RelationshipType::Kinship => {
               // Personality doesn't strongly affect genetic relationships
               base_strength
           }
       };
       
       // Add or update the relationship with the modified strength
       self.add_or_update_relationship(
           target, 
           relationship_type, 
           modified_strength, 
           current_tick
       );
   }
}

/// Get social behavior utility score
pub fn social_behavior_utility(relationships: &SocialNetwork) -> UtilityScore {
    let mut total_utility = 0.0;
    let mut count = 0;
    
    for (_, relationship_map) in &relationships.relationships {
        for (_, relationship) in relationship_map {
            total_utility += relationship.strength.value();
            count += 1;
        }
    }
    
    if count > 0 {
        UtilityScore::new(total_utility / count as f32)
    } else {
        UtilityScore::new(0.0)
    }
}

/// Get the relationship strength between two entities
pub fn get_relationship_strength(
    social_network: &SocialNetwork,
    target: EntityId,
    relationship_type: RelationshipType
) -> Option<RelationshipStrength> {
    social_network.relationships
        .get(&target)
        .and_then(|relationships| relationships.get(&relationship_type))
        .map(|relationship| relationship.strength)
}

// In relationships/social.rs
impl AIModule for SocialNetwork {
    fn update(&mut self) {
        // In a real implementation, this would decay old relationships
        // or update based on recent interactions
    }
    
    fn utility(&self) -> UtilityScore {
        // Calculate overall social interaction utility
        social_behavior_utility(self)
    }
}