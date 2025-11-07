//! Entity tracking - data storage only
//!
//! Stores raw data about tracked entities. No evaluation logic here.
//! Specialized trackers (threat, prey, etc.) read this data and evaluate.

use bevy::prelude::*;
use std::collections::HashMap;

/// Metadata types for tracked entities
#[derive(Debug, Clone)]
pub enum EntityMetadata {
    /// Potential threat (predator, hazard)
    Threat { severity: f32 },

    /// Potential food source
    Prey { attractiveness: f32 },

    /// Social entity (pack member, competitor)
    Social { relationship_strength: f32 },

    /// Neutral/unknown entity
    Neutral,
}

/// Raw data about a tracked entity
#[derive(Debug, Clone)]
pub struct TrackedEntity {
    /// Entity being tracked
    pub entity: Entity,

    /// Last known position
    pub position: Vec2,

    /// Last time this entity was observed (seconds)
    pub last_seen_time: f32,

    /// Distance when last seen
    pub last_distance: f32,

    /// Whether currently in visual contact
    pub in_visual_contact: bool,

    /// Metadata about this entity
    pub metadata: EntityMetadata,
}

impl TrackedEntity {
    pub fn new(entity: Entity, position: Vec2, time: f32, metadata: EntityMetadata) -> Self {
        let distance = 0.0; // Will be updated
        Self {
            entity,
            position,
            last_seen_time: time,
            last_distance: distance,
            in_visual_contact: true,
            metadata,
        }
    }

    /// Time since last observation
    pub fn time_since_seen(&self, current_time: f32) -> f32 {
        current_time - self.last_seen_time
    }
}

/// Component that stores tracked entities (data only, no evaluation)
#[derive(Component, Debug)]
pub struct EntityTracker {
    /// All tracked entities
    tracked: HashMap<Entity, TrackedEntity>,

    /// Maximum entities to track
    max_tracked: usize,
}

impl EntityTracker {
    pub fn new(max_tracked: usize) -> Self {
        Self {
            tracked: HashMap::with_capacity(max_tracked),
            max_tracked,
        }
    }

    /// Add or update tracked entity
    pub fn track_entity(
        &mut self,
        entity: Entity,
        position: Vec2,
        current_time: f32,
        metadata: EntityMetadata,
    ) {
        if let Some(tracked) = self.tracked.get_mut(&entity) {
            // Update existing
            tracked.position = position;
            tracked.last_seen_time = current_time;
            tracked.in_visual_contact = true;
            tracked.metadata = metadata;
        } else {
            // Add new
            self.tracked.insert(
                entity,
                TrackedEntity::new(entity, position, current_time, metadata),
            );
        }
    }

    /// Mark entity as no longer in visual contact
    pub fn lost_visual_contact(&mut self, entity: Entity) {
        if let Some(tracked) = self.tracked.get_mut(&entity) {
            tracked.in_visual_contact = false;
        }
    }

    /// Get tracked entity data
    pub fn get(&self, entity: Entity) -> Option<&TrackedEntity> {
        self.tracked.get(&entity)
    }

    /// Get all tracked entities
    pub fn all(&self) -> impl Iterator<Item = &TrackedEntity> {
        self.tracked.values()
    }

    /// Get entities matching metadata filter
    pub fn filter_by_metadata<F>(&self, predicate: F) -> impl Iterator<Item = &TrackedEntity>
    where
        F: Fn(&EntityMetadata) -> bool,
    {
        self.tracked
            .values()
            .filter(move |t| predicate(&t.metadata))
    }

    /// Remove entities not seen for too long
    pub fn forget_old_entities(&mut self, current_time: f32, forget_after: f32) {
        self.tracked.retain(|_, tracked| {
            tracked.time_since_seen(current_time) < forget_after
        });
    }

    /// Enforce capacity limit (remove least recently seen)
    pub fn enforce_capacity(&mut self) {
        if self.tracked.len() <= self.max_tracked {
            return;
        }

        let mut entries: Vec<_> = self.tracked.iter().map(|(k, v)| (*k, v.last_seen_time)).collect();
        entries.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove oldest
        let to_remove = self.tracked.len() - self.max_tracked;
        let entities_to_remove: Vec<Entity> = entries.iter().take(to_remove).map(|(e, _)| *e).collect();

        for entity in entities_to_remove {
            self.tracked.remove(&entity);
        }
    }
}

impl Default for EntityTracker {
    fn default() -> Self {
        Self::new(20)
    }
}
