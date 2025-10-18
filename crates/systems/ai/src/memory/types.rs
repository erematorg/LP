use crate::core::scorers::Score;
use crate::prelude::*;
use bevy::prelude::*;

/// Memory timestamp (game ticks)
pub type MemoryTimestamp = u64;

/// Types of memory events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryEventType {
    Interaction, // Entity interactions
    Threat,      // Dangerous situations
    Resource,    // Food, shelter, etc.
    Social,      // Group dynamics
}

/// Basic memory event
#[derive(Debug, Clone, Component)]
pub struct MemoryEvent {
    pub timestamp: MemoryTimestamp,
    pub importance: Score, // Importance as Score (0.0-1.0)
    // pub location: Option<Vec2>, // Commented out until terrain is implemented
    pub related_entities: Vec<Entity>,
    pub event_type: MemoryEventType,
}

impl MemoryEvent {
    pub fn new(event_type: MemoryEventType, importance: f32, timestamp: MemoryTimestamp) -> Self {
        Self {
            event_type,
            timestamp,
            importance: Score::new(importance), // Convert to Score
            // location: None,
            related_entities: Vec::new(),
        }
    }

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.related_entities.push(entity);
        self
    }
}

impl AIModule for MemoryEvent {
    fn update(&mut self) {
        // Memories don't need regular updates
        // Could implement decay of importance over time if needed
    }

    fn utility(&self) -> Score {
        // Return importance as utility
        self.importance
    }
}

/// Simple short-term memory for AI decisions
#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct ShortTermMemory {
    pub recent_interactions: Vec<(Entity, RelationshipType, f32)>, // who, what, strength
    pub max_memories: usize,
}

impl ShortTermMemory {
    pub fn new(max_memories: usize) -> Self {
        Self {
            recent_interactions: Vec::new(),
            max_memories,
        }
    }

    pub fn remember_interaction(
        &mut self,
        entity: Entity,
        relationship: RelationshipType,
        strength: f32,
    ) {
        self.recent_interactions
            .push((entity, relationship, strength));
        if self.recent_interactions.len() > self.max_memories {
            self.recent_interactions.remove(0);
        }
    }

    pub fn recall_relationship(&self, entity: Entity) -> Option<f32> {
        self.recent_interactions
            .iter()
            .filter(|(e, _, _)| *e == entity)
            .map(|(_, _, strength)| *strength)
            .next_back() // Most recent interaction (more efficient for DoubleEndedIterator)
    }
}
