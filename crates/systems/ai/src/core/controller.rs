use bevy::prelude::*;
use crate::prelude::*;
use std::f32::consts::PI; //Should make clippy happy yet I may look for another solution later

/// The current state for an Action
#[derive(Debug, Clone, Eq, PartialEq, Component)]
pub enum ActionState {
    Init, Requested, Executing, Cancelled, Success, Failure
}

impl Default for ActionState {
    fn default() -> Self { Self::Init }
}

/// Component for tracking AI behavior
#[derive(Component)]
pub struct AIBehaviorState {
    pub behavior: Behavior,
    pub current_target: Option<Entity>,
    pub current_utility: f32,
    pub action_state: ActionState,
}

impl Default for AIBehaviorState {
    fn default() -> Self {
        Self { behavior: Behavior::Idle, current_target: None, 
               current_utility: 0.0, action_state: ActionState::default() }
    }
}

/// Component for entities that can execute actions
#[derive(Component)]
pub struct ActionExecutorComponent {
    pub executor: Box<dyn ActionExecutor + Send + Sync>,
}

/// Main AI controller component
#[derive(Component)]
pub struct AIController {
    pub perception: Perception,
    pub entity_tracker: EntityTracker,
    pub needs_tracker: NeedsTracker,
    pub modules: Vec<Box<dyn AIModule>>,
    pub personality: Option<Personality>,
    pub social_network: Option<SocialNetwork>,
    pub memories: Vec<MemoryEvent>,
    pub current_tick: MemoryTimestamp,
    pub current_position: Vec2,
}

impl AIController {
    pub fn new(detection_radius: f32, max_tracked_entities: usize) -> Self {
        Self {
            perception: Perception::new(detection_radius),
            entity_tracker: EntityTracker::new(max_tracked_entities),
            needs_tracker: NeedsTracker::new(),
            modules: Vec::new(), personality: None, social_network: None, 
            memories: Vec::new(), current_tick: 0, current_position: Vec2::ZERO,
        }
    }
    
    pub fn with_personality(mut self, personality: Personality) -> Self {
        self.personality = Some(personality); self
    }
    
    pub fn with_social_network(mut self, network: SocialNetwork) -> Self {
        self.social_network = Some(network); self
    }
    
    pub fn register_module(&mut self, module: Box<dyn AIModule>) { self.modules.push(module); }
    pub fn add_need(&mut self, need: Need) { self.needs_tracker.add_need(need); }
    
    pub fn add_memory(&mut self, memory: MemoryEvent) {
        self.memories.push(memory);
        // Sort memories by importance
        self.memories.sort_by(|a, b| b.importance.value().partial_cmp(&a.importance.value()).unwrap());
        
        // Keep only the most important/recent memories
        const MAX_MEMORIES: usize = 50;
        if self.memories.len() > MAX_MEMORIES { self.memories.truncate(MAX_MEMORIES); }
    }
    
    pub fn process_perception_events(&mut self) {
        // Create a snapshot of the current visible entities to avoid borrow issues
        let visible_entities: Vec<(Entity, Vec2, f32)> = 
            self.perception.visible_entities.iter().map(|&(e, p, d)| (e, p, d)).collect();
        let detection_radius = self.perception.detection_radius;
        let highest_threat = self.perception.highest_threat_level;
        
        // Create memories from newly perceived entities
        for (entity, _pos, distance) in &visible_entities {
            let importance = 1.0 - (distance / detection_radius).clamp(0.0, 1.0);
            let is_significant = match self.entity_tracker.get_tracked_entity(*entity) {
                Some(tracked) => tracked.ticks_since_seen > 20, // Haven't seen it in a while
                None => true, // It's completely new
            };
            
            if is_significant && importance > 0.4 {
                self.add_memory(MemoryEvent::new(
                    MemoryEventType::Resource, importance, self.current_tick
                ).with_entity(*entity));
            }
        }
        
        // Create threat memories for high-threat perceptions
        if highest_threat > 0.6 {
            if let Some((entity, _, _)) = self.perception.closest_entity() {
                self.add_memory(MemoryEvent::new(
                    MemoryEventType::Threat, highest_threat, self.current_tick
                ).with_entity(entity));
            }
        }
    }
    
    pub fn update(&mut self, position: Vec2, entities: &[(Entity, Vec2)], time: f32) {
        self.current_position = position;
        self.current_tick += 1;
        
        // Update perception and process events
        self.perception.update(position, entities, time);
        self.process_perception_events();
        
        // Update entity tracker with perception data
        for (entity, pos, distance) in &self.perception.visible_entities {
            let importance = 1.0 - (distance / self.perception.detection_radius).clamp(0.0, 1.0);
            self.entity_tracker.add_entity(*entity, *pos, importance);
        }
        
        // Update all components
        self.entity_tracker.update();
        self.needs_tracker.update();
        for module in &mut self.modules { module.update(); }
        if let Some(ref mut personality) = self.personality { personality.update(); }
        if let Some(ref mut social) = self.social_network { social.update(); }
    }
    
    pub fn select_behavior(&self) -> (Behavior, UtilityScore) {
        let mut behavior_options = Vec::new();
        
        // Add core tracker behaviors
        behavior_options.push((&self.perception as &dyn AIModule, 
                              self.perception.utility(),
                              map_perception_to_behavior(&self.perception)));
        behavior_options.push((&self.entity_tracker as &dyn AIModule,
                              self.entity_tracker.utility(),
                              map_entity_tracker_to_behavior(&self.entity_tracker)));
        behavior_options.push((&self.needs_tracker as &dyn AIModule,
                              self.needs_tracker.utility(),
                              map_needs_to_behavior(&self.needs_tracker)));
        
        // Add personality-driven behaviors
        if let Some(ref personality) = self.personality {
            behavior_options.push((personality as &dyn AIModule, 
                                  personality.attack_utility(), Behavior::Fight));
            behavior_options.push((personality as &dyn AIModule,
                                  personality.flee_utility(), Behavior::Flee));
        }
        
        // Add custom modules and social behaviors
        for module in &self.modules {
            behavior_options.push((module.as_ref(), module.utility(), 
                                  self.map_module_to_behavior(module.as_ref())));
        }
        if let Some(ref social) = self.social_network {
            behavior_options.push((social as &dyn AIModule, social.utility(), Behavior::Socialize));
        }
        
        determine_behavior(&behavior_options)
    }
    
    // Maps a module to an appropriate behavior based on type
    fn map_module_to_behavior(&self, _module: &dyn AIModule) -> Behavior { Behavior::Idle }
}

/// System to update AI behaviors
pub fn update_ai_behavior_system(mut query: Query<(&AIController, &mut AIBehaviorState)>) {
    for (controller, mut state) in query.iter_mut() {
        let (behavior, utility) = controller.select_behavior();
        state.behavior = behavior;
        state.current_utility = utility.value();
    }
}

/// System to execute AI behaviors
pub fn execute_ai_behavior_system(
    mut query: Query<(&mut AIBehaviorState, &AIController)>,
    mut executor_query: Query<&mut ActionExecutorComponent>
) {
    if let Ok(mut executor_component) = executor_query.get_single_mut() {
        for (mut state, controller) in query.iter_mut() {
            execute_behavior(&mut state, controller, &mut executor_component.executor);
        }
    }
}

/// Execute the current behavior with state machine handling
fn execute_behavior(
    state: &mut AIBehaviorState, 
    controller: &AIController,
    executor: &mut Box<dyn ActionExecutor + Send + Sync>
) {
    match state.action_state {
        ActionState::Init => { state.action_state = ActionState::Requested; },
        ActionState::Requested => {
            state.action_state = ActionState::Executing;
            
            // Perform initial action setup based on behavior
            match state.behavior {
                Behavior::Hunt => {
                    if let Some((entity, pos, _)) = controller.perception.closest_entity() {
                        state.current_target = Some(entity);
                        executor.move_toward(pos, 1.0);
                    }
                },
                Behavior::Flee => {
                    if let Some((_, pos, _)) = controller.perception.closest_entity() {
                        executor.flee_from(pos);
                    }
                },
                Behavior::Fight => {
                    let target = state.current_target.or_else(|| 
                        controller.perception.closest_entity().map(|(e, _, _)| e));
                    executor.attack(target);
                },
                Behavior::Rest => { executor.idle(1.0); },
                Behavior::Socialize => {
                    if let Some((entity, pos, _)) = controller.perception.closest_entity() {
                        state.current_target = Some(entity);
                        executor.move_toward(pos, 0.5);
                    }
                },
                Behavior::Explore => {
                    if let Some((_, pos, _)) = controller.perception.closest_entity() {
                        let angle = (controller.current_tick as f32 * 0.1).sin() * PI;
                        let random_pos = Vec2::new(angle.cos(), angle.sin()) * 100.0 + pos;
                        executor.move_toward(random_pos, 0.7);
                    }
                },
                Behavior::Idle => { executor.idle(0.5); }
            }
        },
        ActionState::Executing => {
            // Continue execution based on behavior
            let completed = match state.behavior {
                Behavior::Hunt => {
                    if let Some((entity, pos, _)) = controller.perception.closest_entity() {
                        state.current_target = Some(entity);
                        !executor.move_toward(pos, 1.0)
                    } else { true }
                },
                Behavior::Flee => {
                    if let Some((_, pos, _)) = controller.perception.closest_entity() {
                        !executor.flee_from(pos)
                    } else { true }
                },
                Behavior::Fight => {
                    let target = state.current_target.or_else(|| 
                        controller.perception.closest_entity().map(|(e, _, _)| e));
                    executor.attack(target)
                },
                Behavior::Rest => { executor.idle(1.0) },
                Behavior::Socialize => {
                    if let Some((entity, pos, _)) = controller.perception.closest_entity() {
                        state.current_target = Some(entity);
                        !executor.move_toward(pos, 0.5)
                    } else { true }
                },
                Behavior::Explore => {
                    if let Some((_, pos, _)) = controller.perception.closest_entity() {
                        let angle = (controller.current_tick as f32 * 0.1).sin() * PI;
                        let random_pos = Vec2::new(angle.cos(), angle.sin()) * 100.0 + pos;
                        !executor.move_toward(random_pos, 0.7)
                    } else { true }
                },
                Behavior::Idle => { executor.idle(0.5) }
            };
            
            if completed { state.action_state = ActionState::Success; }
        },
        ActionState::Cancelled => {
            executor.cleanup();
            state.action_state = ActionState::Failure;
        },
        ActionState::Success | ActionState::Failure => {
            state.action_state = ActionState::Init;
            state.current_target = None;
        }
    }
}

// Helper functions to map module states to behaviors
fn map_perception_to_behavior(perception: &Perception) -> Behavior {
    if perception.highest_threat_level > 0.7 { Behavior::Flee }
    else if perception.highest_threat_level > 0.4 { Behavior::Fight }
    else { Behavior::Explore }
}

fn map_entity_tracker_to_behavior(tracker: &EntityTracker) -> Behavior {
    match tracker.get_most_important_entity() {
        Some((_, entity)) if entity.importance > 0.7 => Behavior::Hunt,
        Some(_) => Behavior::Explore,
        None => Behavior::Idle
    }
}

fn map_needs_to_behavior(needs: &NeedsTracker) -> Behavior {
    match needs.get_most_urgent_need() {
        Some((NeedType::Hunger, _)) => Behavior::Hunt,
        Some((NeedType::Safety, _)) => Behavior::Flee,
        Some((NeedType::Rest, _)) => Behavior::Rest,
        Some((NeedType::Social, _)) => Behavior::Socialize,
        None => Behavior::Idle
    }
}