use crate::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// Entity identifier type (compatible with Bevy ECS)
pub type EntityId = Entity;

/// Normalized relationship value between entities
#[derive(Debug, Clone, Copy, Component, Reflect)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
pub enum RelationshipType {
    Cooperation, // Mutual benefit
    Competition, // Resource rivals
    Predation,   // Hunter/prey dynamics
    Fear,        // Threat response
    Kinship,     // Family/genetic bonds
}

/// Single relationship between two entities
#[derive(Debug, Clone, Component, Reflect)]
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
#[derive(Debug, Default, Component, Reflect)]
pub struct SocialNetwork {
    relationships: HashMap<EntityId, HashMap<RelationshipType, EntityRelationship>>,
}

impl SocialNetwork {
    /// Add a new relationship or update an existing one
    pub fn add_or_update_relationship(
        &mut self,
        target: EntityId,
        relationship_type: RelationshipType,
        strength: f32,
        current_tick: u64,
    ) {
        let mut relationship = EntityRelationship {
            strength: RelationshipStrength::new(strength),
            relationship_type,
            last_interaction_tick: current_tick,
        };

        // If relationship already exists, update based on existing interaction history
        if let Some(existing_relationship) = self
            .relationships
            .entry(target)
            .or_default()
            .get_mut(&relationship_type)
        {
            // Modify strength based on interaction frequency
            relationship.strength.adjust(
                (current_tick - existing_relationship.last_interaction_tick) as f32 / 1000.0,
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
        min_strength: Option<f32>,
    ) -> Vec<(EntityId, &EntityRelationship)> {
        let mut result = Vec::new();

        for (&entity_id, relationships) in &self.relationships {
            for (current_type, relationship) in relationships {
                let type_match = relationship_type.is_none_or(|rt| *current_type == rt);
                let strength_match =
                    min_strength.is_none_or(|ms| relationship.strength.value() >= ms);

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
        current_tick: u64,
    ) {
        // Base strength from existing relationship or default
        let base_strength = self
            .relationships
            .get(&target)
            .and_then(|r| r.get(&relationship_type))
            .map_or(0.5, |rel| rel.strength.value());

        // Apply personality modifiers
        let modified_strength = match relationship_type {
            RelationshipType::Cooperation => {
                // More competitive personalities are less cooperative
                base_strength * (1.0 - personality.competitive_strength * 0.3)
            }
            RelationshipType::Competition => {
                // More resource assertive personalities compete more strongly
                base_strength * (1.0 + personality.resource_assertiveness * 0.5)
            }
            RelationshipType::Predation => {
                // More assertive and competitive personalities have stronger predation
                base_strength
                    * (1.0
                        + personality.resource_assertiveness * 0.6
                        + personality.competitive_strength * 0.4)
            }
            RelationshipType::Fear => {
                // Less stress tolerant personalities feel more fear
                base_strength * (1.0 + (1.0 - personality.stress_tolerance) * 0.7)
            }
            RelationshipType::Kinship => {
                // Personality doesn't strongly affect genetic relationships
                base_strength
            }
        };

        // Add or update the relationship with the modified strength
        self.add_or_update_relationship(target, relationship_type, modified_strength, current_tick);
    }
}

/// Get social behavior utility score
pub fn social_behavior_utility(relationships: &SocialNetwork) -> UtilityScore {
    let mut total_utility = 0.0;
    let mut count = 0;

    for relationship_map in relationships.relationships.values() {
        for relationship in relationship_map.values() {
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
    relationship_type: RelationshipType,
) -> Option<RelationshipStrength> {
    social_network
        .relationships
        .get(&target)
        .and_then(|relationships| relationships.get(&relationship_type))
        .map(|relationship| relationship.strength)
}

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

// Optional Relations-based components (alternative to HashMap approach)
#[derive(Component, Debug, Clone, Reflect)]
pub struct SocialRelation {
    pub target: Entity,
    pub strength: RelationshipStrength,
    pub relationship_type: RelationshipType,
    pub last_interaction_tick: u64,
    /// Spatial distance when relationship was last updated (for proximity influence)
    pub last_distance: Option<f32>,
}

impl SocialRelation {
    /// Adjust relationship strength based on spatial proximity
    /// Closer entities have stronger social bonds (universal for all life)
    pub fn update_with_proximity(&mut self, current_distance: f32, proximity_influence: f32) {
        if let Some(last_dist) = self.last_distance {
            // Getting closer strengthens bonds, getting farther weakens them
            let distance_change = last_dist - current_distance;
            let proximity_modifier = distance_change * proximity_influence * 0.01;
            self.strength.adjust(proximity_modifier);
        }
        self.last_distance = Some(current_distance);
    }

    /// Calculate collective utility influence from nearby entities
    /// Used for swarm intelligence patterns - closer entities have more influence
    pub fn proximity_utility_modifier(&self, max_influence_distance: f32) -> f32 {
        if let Some(distance) = self.last_distance {
            let proximity_factor = (max_influence_distance - distance) / max_influence_distance;
            proximity_factor.max(0.0) * self.strength.value()
        } else {
            0.0
        }
    }
}
