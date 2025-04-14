use bevy::prelude::*;
use crate::core::utility::{Behavior, UtilityScore, determine_behavior};
use crate::core::interfaces::{AIModule, ActionExecutor};
use crate::personality::traits::Personality;
use crate::relationships::social::SocialNetwork;
use crate::drives::needs::{Need, NeedType};
use crate::memory::types::{MemoryEvent, MemoryTimestamp};
use crate::trackers::prelude::*;
pub struct AIController {
    // Core AI components
    pub perception: Perception,
    pub modules: Vec<Box<dyn AIModule>>,
    pub behavior: Behavior,
    
    // Entity components
    pub personality: Option<Personality>,
    pub social_network: Option<SocialNetwork>,
    pub needs_tracker: Option<NeedsTracker>,
    pub memories: Vec<MemoryEvent>,
    
    // Current state
    pub current_target: Option<Entity>,
    pub current_utility: f32,
    pub current_tick: MemoryTimestamp,
}

impl AIController {
    pub fn new(detection_radius: f32) -> Self {
        Self {
            perception: Perception::new(detection_radius),
            modules: Vec::new(),
            behavior: Behavior::Idle,
            personality: None,
            social_network: None,
            needs_tracker: None,
            memories: Vec::new(),
            current_target: None,
            current_utility: 0.0,
            current_tick: 0,
        }
    }
    
    pub fn with_personality(mut self, personality: Personality) -> Self {
        self.personality = Some(personality);
        self
    }
    
    pub fn with_social_network(mut self, network: SocialNetwork) -> Self {
        self.social_network = Some(network);
        self
    }
    
    pub fn with_needs_tracker(mut self, tracker: NeedsTracker) -> Self {
        self.needs_tracker = Some(tracker);
        self
    }
    
    pub fn add_memory(&mut self, memory: MemoryEvent) {
        self.memories.push(memory);
    }
    
    pub fn register_module(&mut self, module: Box<dyn AIModule>) {
        self.modules.push(module);
    }
    
    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        // Increment tick counter
        self.current_tick += 1;
        
        // 1. Update perception
        self.perception.update(position, entities, time);
        
        // 2. Update all modules
        for module in &mut self.modules {
            module.update();
        }
        
        // Also update integrated components
        if let Some(ref mut personality) = self.personality {
            personality.update();
        }
        
        if let Some(ref mut social) = self.social_network {
            social.update();
        }
        
        if let Some(ref mut needs) = self.needs_tracker {
            needs.update();
        }
        
        // 3. Collect modules with behaviors and utilities
        let mut behavior_options = Vec::new();
        
        // Add each module with appropriate behavior mapping
        for module in &self.modules {
            // This would be more sophisticated in a real implementation
            let behavior = self.map_module_to_behavior(module.as_ref());
            behavior_options.push((
                module.as_ref() as &dyn AIModule,
                module.utility(),
                behavior
            ));
        }
        
        // Add personality-driven behaviors
        if let Some(ref personality) = self.personality {
            // Attack behavior based on personality
            behavior_options.push((
                personality as &dyn AIModule,
                personality.attack_utility(),
                Behavior::Fight
            ));
            
            // Flee behavior based on personality
            behavior_options.push((
                personality as &dyn AIModule,
                personality.flee_utility(),
                Behavior::Flee
            ));
        }
        
        // Add needs-driven behaviors
        if let Some(ref needs) = self.needs_tracker {
            if let Some((need_type, urgency)) = needs.get_most_urgent_need() {
                // Map need type to behavior
                let behavior = match need_type {
                    NeedType::Hunger => Behavior::Hunt,
                    NeedType::Safety => Behavior::Flee,
                    NeedType::Rest => Behavior::Rest,
                    NeedType::Social => Behavior::Socialize,
                };
                
                behavior_options.push((
                    needs as &dyn AIModule,
                    urgency,
                    behavior
                ));
            }
        }
        
        // Add social behaviors
        if let Some(ref social) = self.social_network {
            behavior_options.push((
                social as &dyn AIModule,
                social.utility(),
                Behavior::Socialize
            ));
        }
        
        // 4. Select behavior with highest utility
        let (new_behavior, utility) = determine_behavior(&behavior_options);
        self.behavior = new_behavior;
        self.current_utility = utility.value();
    }
    
    // Maps a module to an appropriate behavior based on type
    fn map_module_to_behavior(&self, module: &dyn AIModule) -> Behavior {
        // Default mapping - would be enhanced based on module type
        Behavior::Idle
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
            Behavior::Fight => {
                if let Some(target) = self.current_target {
                    executor.attack(Some(target));
                } else if let Some((entity, _, _)) = self.perception.closest_entity() {
                    executor.attack(Some(entity));
                }
            },
            Behavior::Rest => {
                executor.idle(1.0);
            },
            Behavior::Socialize => {
                if let Some((entity, pos, _)) = self.perception.closest_entity() {
                    self.current_target = Some(entity);
                    executor.move_toward(pos, 0.5);
                }
            },
            Behavior::Explore => {
                // Simple exploration by moving in a random direction
                // Would be more sophisticated in a real implementation
                if let Some((_, pos, _)) = self.perception.closest_entity() {
                    let angle = (self.current_tick as f32 * 0.1).sin() * 3.14;
                    let random_pos = Vec2::new(angle.cos(), angle.sin()) * 100.0 + pos;
                    executor.move_toward(random_pos, 0.7);
                }
            },
            Behavior::Idle | _ => {
                executor.idle(0.5);
            }
        }
    }
}