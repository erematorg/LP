use bevy::prelude::*;
use crate::prelude::*;


/// Memory timestamp (game ticks)
pub type MemoryTimestamp = u64;

/// Types of memory events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryEventType {
    Interaction,  // Entity interactions
    Threat,       // Dangerous situations
    Resource,     // Food, shelter, etc.
    Social,       // Group dynamics
}

/// Basic memory event
#[derive(Debug, Clone, Component)]
pub struct MemoryEvent {
    pub timestamp: MemoryTimestamp,
    pub importance: UtilityScore,  // Use UtilityScore instead of raw f32
    // pub location: Option<Vec2>, // Commented out until terrain is implemented
    pub related_entities: Vec<Entity>,
    pub event_type: MemoryEventType,
}

impl MemoryEvent {
    pub fn new(event_type: MemoryEventType, importance: f32, timestamp: MemoryTimestamp) -> Self {
        Self {
            event_type,
            timestamp,
            importance: UtilityScore::new(importance),  // Convert to UtilityScore
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
    
    fn utility(&self) -> UtilityScore {
        // Return importance as utility
        self.importance
    }
}