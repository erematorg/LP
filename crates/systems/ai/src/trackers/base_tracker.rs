use crate::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// Core tracker for monitoring entities
pub struct EntityTracker {
    /// Entities being tracked
    tracked_entities: HashMap<Entity, TrackedEntity>,
    /// Maximum number of entities to track
    max_tracked_entities: usize,
    /// Priority score for decision making
    priority: f32,
}

/// Information about a tracked entity
pub struct TrackedEntity {
    /// Last known position
    pub last_seen_position: Vec2,
    /// Whether the entity is currently visible
    pub visual_contact: bool,
    /// Ticks since last seen
    pub ticks_since_seen: u32,
    /// Importance of this entity (0.0-1.0)
    pub importance: f32,
}

impl AIModule for EntityTracker {
    fn update(&mut self) {
        // Update tracking information
        for tracked in self.tracked_entities.values_mut() {
            if !tracked.visual_contact {
                tracked.ticks_since_seen += 1;

                // Reduce importance over time when not seen
                if tracked.ticks_since_seen > 10 {
                    tracked.importance *= 0.99;
                }
            }
        }

        // Calculate priority based on most important tracked entity
        self.priority = self
            .tracked_entities
            .values()
            .map(|entity| entity.importance)
            .fold(0.0, f32::max);

        // Remove low importance entities when over capacity
        if self.tracked_entities.len() > self.max_tracked_entities {
            let mut entries: Vec<_> = self.tracked_entities.iter().collect();
            entries.sort_by(|a, b| b.1.importance.partial_cmp(&a.1.importance).unwrap());

            let cutoff = entries[self.max_tracked_entities].1.importance;
            self.tracked_entities.retain(|_, e| e.importance > cutoff);
        }
    }

    fn utility(&self) -> UtilityScore {
        UtilityScore::new(self.priority)
    }
}

impl EntityTracker {
    pub fn new(max_tracked_entities: usize) -> Self {
        Self {
            tracked_entities: HashMap::new(),
            max_tracked_entities,
            priority: 0.0,
        }
    }

    pub fn add_entity(&mut self, entity: Entity, position: Vec2, importance: f32) {
        self.tracked_entities.insert(
            entity,
            TrackedEntity {
                last_seen_position: position,
                visual_contact: true,
                ticks_since_seen: 0,
                importance: importance.clamp(0.0, 1.0),
            },
        );
    }

    pub fn update_entity(&mut self, entity: Entity, position: Vec2) {
        if let Some(tracked) = self.tracked_entities.get_mut(&entity) {
            tracked.last_seen_position = position;
            tracked.visual_contact = true;
            tracked.ticks_since_seen = 0;
        }
    }

    pub fn get_tracked_entity(&self, entity: Entity) -> Option<&TrackedEntity> {
        self.tracked_entities.get(&entity)
    }

    pub fn get_most_important_entity(&self) -> Option<(Entity, &TrackedEntity)> {
        self.tracked_entities
            .iter()
            .max_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap())
            .map(|(e, data)| (*e, data))
    }
}
