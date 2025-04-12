use bevy::prelude::*;
use crate::core::interfaces::AIModule;
use crate::core::utility::UtilityScore;
use std::collections::HashMap;

/// Core tracker for monitoring entities
pub struct EntityTracker {
    /// Entities being tracked
    tracked_entities: HashMap<Entity, TrackedEntity>,
    /// Maximum number of entities to track
    max_tracked_entities: usize,
}

/// Information about a tracked entity
pub struct TrackedEntity {
    /// Last known position
    pub last_seen_position: Vec2,
    /// Whether the entity is currently visible
    pub visual_contact: bool,
    /// Ticks since last seen
    pub ticks_since_seen: u32,
}

impl AIModule for EntityTracker {
    fn update(&mut self) {
        // Update tracking information
    }

    fn utility(&self) -> UtilityScore {
        UtilityScore::new(0.0)
    }
}

impl EntityTracker {
    pub fn new(max_tracked_entities: usize) -> Self {
        Self {
            tracked_entities: HashMap::new(),
            max_tracked_entities,
        }
    }
}