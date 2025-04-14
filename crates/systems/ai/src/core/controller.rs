// In controller.rs
use bevy::prelude::*;
use crate::core::utility::{Behavior, determine_behavior};
use crate::core::interfaces::{AIModule, ActionExecutor};
use crate::trackers::perception_tracker::Perception;

pub struct AIController {
    // Core AI components
    pub perception: Perception,
    pub modules: Vec<Box<dyn AIModule>>,
    pub behavior: Behavior,
    
    // Current state
    pub current_target: Option<Entity>,
    pub current_utility: f32,
}

impl AIController {
    pub fn new(detection_radius: f32) -> Self {
        Self {
            perception: Perception::new(detection_radius),
            modules: Vec::new(),
            behavior: Behavior::Idle,
            current_target: None,
            current_utility: 0.0,
        }
    }
    
    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        // 1. Update perception
        self.perception.update(position, entities, time);
        
        // 2. Update all modules
        for module in &mut self.modules {
            module.update();
        }
        
        // 3. Collect modules with behaviors and utilities
        let mut behavior_options = Vec::new();
        
        // Add each module with its behavior association
        for module in &self.modules {
            // This is simplified - you'd actually map modules to appropriate behaviors
            behavior_options.push((
                module.as_ref() as &dyn AIModule,
                module.utility(),
                Behavior::Idle // Replace with actual behavior mapping
            ));
        }
        
        // 4. Select behavior with highest utility
        let (new_behavior, utility) = determine_behavior(&behavior_options);
        self.behavior = new_behavior;
        self.current_utility = utility.value();
    }
    
    pub fn execute(&mut self, executor: &mut dyn ActionExecutor) {
        // Execute action based on current behavior
        match self.behavior {
            Behavior::Hunt => {
                if let Some((entity, pos, _)) = self.perception.closest_entity() {
                    self.current_target = Some(entity);
                    executor.move_toward(pos, 1.0);
                }
            },
            Behavior::Flee => {
                if let Some((_, pos, _)) = self.perception.closest_entity() {
                    executor.flee_from(pos);
                }
            },
            Behavior::Idle => {
                executor.idle(1.0);
            },
            _ => {
                // Handle other behaviors
                executor.idle(0.5);
            }
        }
    }
}