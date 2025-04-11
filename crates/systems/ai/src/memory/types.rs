use bevy::prelude::*;

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
    pub importance: f32,  // 0.0-1.0 value
    // pub location: Option<Vec2>,  // Commented out until terrain is implemented
    pub related_entities: Vec<Entity>,
    pub event_type: MemoryEventType,
}

impl MemoryEvent {
    pub fn new(event_type: MemoryEventType, importance: f32, timestamp: MemoryTimestamp) -> Self {
        Self {
            event_type,
            timestamp,
            importance: importance.clamp(0.0, 1.0),
            // location: None,
            related_entities: Vec::new(),
        }
    }
    
    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.related_entities.push(entity);
        self
    }
}