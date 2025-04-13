use bevy::prelude::*;

pub struct Perception {
    pub visible_entities: Vec<(Entity, Vec2, f32)>, // Entity, position, distance
    pub detection_radius: f32,
    pub last_updated: f32,
}

impl Perception {
    pub fn new(detection_radius: f32) -> Self {
        Self { 
            visible_entities: Vec::new(),
            detection_radius,
            last_updated: 0.0
        }
    }
    
    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        self.visible_entities.clear();
        self.last_updated = time;
        
        for (entity, entity_pos) in entities {
            let distance = position.distance(*entity_pos);
            if distance <= self.detection_radius {
                self.visible_entities.push((*entity, *entity_pos, distance));
            }
        }
    }
    
    pub fn closest_entity(&self) -> Option<(Entity, Vec2, f32)> {
        self.visible_entities.iter()
            .min_by(|(_, _, dist_a), (_, _, dist_b)| 
                dist_a.partial_cmp(dist_b).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
    }
}