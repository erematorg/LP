use bevy::prelude::*;
use crate::prelude::*;
use std::f32::consts::PI; //Should make clippy happy yet I may look for another solution later
pub struct AIController {
    // Core AI components
    pub perception: Perception,
    pub entity_tracker: EntityTracker,
    pub needs_tracker: NeedsTracker,
    pub modules: Vec<Box<dyn AIModule>>,
    pub behavior: Behavior,
    
    // Entity components
    pub personality: Option<Personality>,
    pub social_network: Option<SocialNetwork>,
    pub memories: Vec<MemoryEvent>,
    
    // Current state
    pub current_target: Option<Entity>,
    pub current_utility: f32,
    pub current_tick: MemoryTimestamp,
    pub current_position: Vec2,
}

impl AIController {
    pub fn new(detection_radius: f32, max_tracked_entities: usize) -> Self {
        let perception = Perception::new(detection_radius);
        let entity_tracker = EntityTracker::new(max_tracked_entities);
        let needs_tracker = NeedsTracker::new();
        
        Self {
            perception,
            entity_tracker,
            needs_tracker,
            modules: Vec::new(),
            behavior: Behavior::Idle,
            personality: None,
            social_network: None,
            memories: Vec::new(),
            current_target: None,
            current_utility: 0.0,
            current_tick: 0,
            current_position: Vec2::ZERO,
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
    
    pub fn register_module(&mut self, module: Box<dyn AIModule>) {
        self.modules.push(module);
    }
    
    pub fn add_memory(&mut self, memory: MemoryEvent) {
        self.memories.push(memory);
        
        // Sort memories by importance
        self.memories.sort_by(|a, b| b.importance.value().partial_cmp(&a.importance.value()).unwrap());
        
        // Keep only the most important/recent memories
        const MAX_MEMORIES: usize = 50;
        if self.memories.len() > MAX_MEMORIES {
            self.memories.truncate(MAX_MEMORIES);
        }
    }
    
    pub fn add_need(&mut self, need: Need) {
        self.needs_tracker.add_need(need);
    }
    
    pub fn process_perception_events(&mut self) {
        // Store visible entities in a temporary vector to avoid borrow issues
        let visible_entities: Vec<(Entity, Vec2, f32)> = self.perception.visible_entities.clone();
        let highest_threat = self.perception.highest_threat_level;
        
        // Create memories from newly perceived entities
        for (entity, _pos, distance) in &visible_entities {
            // Calculate importance based on proximity
            let importance = 1.0 - (*distance / self.perception.detection_radius).clamp(0.0, 1.0);
            
            // Check if this is a new entity or one we haven't seen in a while
            let is_significant = if let Some(tracked) = self.entity_tracker.get_tracked_entity(*entity) {
                tracked.ticks_since_seen > 20  // We haven't seen it for a while
            } else {
                true  // It's completely new
            };
            
            // Only create memories for significant perception events
            if is_significant && importance > 0.4 {
                let memory = MemoryEvent::new(
                    MemoryEventType::Resource,  // Default type, would be based on entity type
                    importance, 
                    self.current_tick
                )
                .with_entity(*entity);
                
                self.add_memory(memory);
            }
        }
        
        // Create threat memories for high-threat perceptions
        if highest_threat > 0.6 {
            if let Some((entity, _, _)) = self.perception.closest_entity() {
                let threat_memory = MemoryEvent::new(
                    MemoryEventType::Threat,
                    highest_threat,
                    self.current_tick
                )
                .with_entity(entity);
                
                self.add_memory(threat_memory);
            }
        }
    }
    
    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        // Update current position
        self.current_position = position;
        
        // Increment tick counter
        self.current_tick += 1;
        
        // 1. Update perception and entity tracking
        self.perception.update(position, entities, time);
        
        // Process perception events into memories
        self.process_perception_events();
        
        // Update entity tracker with perception data
        for (entity, pos, distance) in &self.perception.visible_entities {
            // Calculate importance based on proximity - closer entities are more important
            let importance = 1.0 - (*distance / self.perception.detection_radius).clamp(0.0, 1.0);
            self.entity_tracker.add_entity(*entity, *pos, importance);
        }
        
        // 2. Update all core trackers
        // We call AIModule update() which is separate from specific tracker update methods
        self.entity_tracker.update();
        self.needs_tracker.update();
        
        // 3. Update all other modules
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
        
        // 4. Collect options for behavior selection
        let mut behavior_options = Vec::new();
        
        // Add core trackers
        behavior_options.push((
            &self.perception as &dyn AIModule,
            self.perception.utility(),
            map_perception_to_behavior(&self.perception)
        ));
        
        behavior_options.push((
            &self.entity_tracker as &dyn AIModule,
            self.entity_tracker.utility(),
            map_entity_tracker_to_behavior(&self.entity_tracker)
        ));
        
        behavior_options.push((
            &self.needs_tracker as &dyn AIModule,
            self.needs_tracker.utility(),
            map_needs_to_behavior(&self.needs_tracker)
        ));
        
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
        
        // Add custom modules
        for module in &self.modules {
            behavior_options.push((
                module.as_ref(),
                module.utility(),
                self.map_module_to_behavior(module.as_ref())
            ));
        }
        
        // Add social behaviors
        if let Some(ref social) = self.social_network {
            behavior_options.push((
                social as &dyn AIModule,
                social.utility(),
                Behavior::Socialize
            ));
        }
        
        // 5. Select behavior with highest utility
        let (new_behavior, utility) = determine_behavior(&behavior_options);
        self.behavior = new_behavior;
        self.current_utility = utility.value();
    }
    
    // Maps a module to an appropriate behavior based on type
    fn map_module_to_behavior(&self, _module: &dyn AIModule) -> Behavior {
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
                if let Some((_, pos, _)) = self.perception.closest_entity() {
                    let angle = (self.current_tick as f32 * 0.1).sin() * PI;
                    let random_pos = Vec2::new(angle.cos(), angle.sin()) * 100.0 + pos;
                    executor.move_toward(random_pos, 0.7);
                }
            },
            Behavior::Idle => {
                executor.idle(0.5);
            }
        }
    }
}

// Helper functions to map module states to behaviors
fn map_perception_to_behavior(perception: &Perception) -> Behavior {
    if perception.highest_threat_level > 0.7 {
        Behavior::Flee
    } else if perception.highest_threat_level > 0.4 {
        Behavior::Fight
    } else {
        Behavior::Explore
    }
}

fn map_entity_tracker_to_behavior(tracker: &EntityTracker) -> Behavior {
    if let Some((_, entity)) = tracker.get_most_important_entity() {
        if entity.importance > 0.7 {
            Behavior::Hunt
        } else {
            Behavior::Explore
        }
    } else {
        Behavior::Idle
    }
}

fn map_needs_to_behavior(needs: &NeedsTracker) -> Behavior {
    if let Some((need_type, _)) = needs.get_most_urgent_need() {
        match need_type {
            NeedType::Hunger => Behavior::Hunt,
            NeedType::Safety => Behavior::Flee,
            NeedType::Rest => Behavior::Rest,
            NeedType::Social => Behavior::Socialize,
        }
    } else {
        Behavior::Idle
    }
}