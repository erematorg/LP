use crate::prelude::*;
use bevy::prelude::*;

pub struct Perception {
    pub visible_entities: Vec<(Entity, Vec2, f32)>, // Entity, position, distance
    pub detection_radius: f32,
    pub last_updated: f32,
    pub highest_threat_level: f32, // 0.0-1.0 threat level
}

impl Perception {
    pub fn new(detection_radius: f32) -> Self {
        Self {
            visible_entities: Vec::new(),
            detection_radius,
            last_updated: 0.0,
            highest_threat_level: 0.0,
        }
    }

    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        self.visible_entities.clear();
        self.last_updated = time;
        self.highest_threat_level = 0.0;

        for (entity, entity_pos) in entities {
            let distance = position.distance(*entity_pos);
            if distance <= self.detection_radius {
                // Basic threat calculation - closer entities are more threatening
                let threat_level = 1.0 - (distance / self.detection_radius);
                self.highest_threat_level = self.highest_threat_level.max(threat_level);
                self.visible_entities.push((*entity, *entity_pos, distance));
            }
        }
    }

    pub fn closest_entity(&self) -> Option<(Entity, Vec2, f32)> {
        self.visible_entities
            .iter()
            .min_by(|(_, _, dist_a), (_, _, dist_b)| {
                dist_a
                    .partial_cmp(dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
}

impl AIModule for Perception {
    fn update(&mut self) {
        // This would be called from controller - actual update happens in update() method
        // with position and entities data

        // Decay threat level over time when not explicitly updated
        self.highest_threat_level *= 0.95;
    }

    fn utility(&self) -> UtilityScore {
        // Return threat level as utility - higher threat means more important
        UtilityScore::new(self.highest_threat_level)
    }
}
