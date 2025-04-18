use crate::prelude::*;
use bevy::prelude::*;

/// State machine for action execution
#[derive(Debug, Clone, Eq, PartialEq, Component)]
pub enum ActionState {
    Init,        // Initial state
    Requested,   // Action has been requested
    Executing,   // Action is currently executing
    Cancelled,   // Action was cancelled and needs cleanup
    Success,     // Action completed successfully
    Failure      // Action failed to complete
}

impl Default for ActionState {
    fn default() -> Self { Self::Init }
}

/// Context provided to actions for execution
pub struct ActionContext<'a> {
    pub entity: Entity,
    pub transform: &'a mut Transform,
    pub perception: &'a Perception,
    pub entity_tracker: &'a EntityTracker,
    pub executor: &'a mut Box<dyn ActionExecutor + Send + Sync>,
    pub current_position: Vec2,
    pub target: Option<Entity>,
    pub delta_time: f32,
}

/// Core trait for all executable actions
pub trait Action: Send + Sync {
    /// Execute one step of the action
    fn execute(&mut self, context: &mut ActionContext) -> ActionState;
    
    /// Called when the action is starting (transitioning from Requested to Executing)
    /// Return false if the action can't start for some reason
    fn on_start(&mut self, _context: &mut ActionContext) -> bool {
        true // Default implementation always succeeds
    }
    
    /// Called when the action is cancelled
    fn on_cancel(&mut self, context: &mut ActionContext) {
        context.executor.cleanup();
    }
    
    /// Called when the action completes successfully
    fn on_success(&mut self, _context: &mut ActionContext) {}
    
    /// Called when the action fails
    fn on_failure(&mut self, _context: &mut ActionContext) {}
    
    /// Label for debugging and tracing
    fn label(&self) -> &str {
        "Unnamed Action"
    }
}

/// Composite action that executes a sequence of steps
pub struct Sequence {
    steps: Vec<Box<dyn Action>>,
    current_step: usize,
    step_state: ActionState,
}

impl Sequence {
    pub fn new(steps: Vec<Box<dyn Action>>) -> Self {
        Self {
            steps,
            current_step: 0,
            step_state: ActionState::Init,
        }
    }
}

impl Action for Sequence {
    fn execute(&mut self, context: &mut ActionContext) -> ActionState {
        // Return success if we've completed all steps
        if self.current_step >= self.steps.len() {
            return ActionState::Success;
        }
        
        // Execute current step
        match self.step_state {
            ActionState::Init => {
                self.step_state = ActionState::Requested;
                ActionState::Executing
            },
            ActionState::Requested | ActionState::Executing => {
                self.step_state = self.steps[self.current_step].execute(context);
                
                // Handle step completion
                match self.step_state {
                    ActionState::Success => {
                        // Call on_success for the completed step
                        self.steps[self.current_step].on_success(context);
                        
                        self.current_step += 1;
                        if self.current_step >= self.steps.len() {
                            ActionState::Success
                        } else {
                            self.step_state = ActionState::Init;
                            ActionState::Executing
                        }
                    },
                    ActionState::Failure => {
                        // Call on_failure for the failed step
                        self.steps[self.current_step].on_failure(context);
                        ActionState::Failure
                    },
                    _ => ActionState::Executing
                }
            },
            ActionState::Cancelled => {
                self.steps[self.current_step].on_cancel(context);
                ActionState::Failure
            },
            ActionState::Success | ActionState::Failure => {
                // This shouldn't happen but just in case
                if self.current_step + 1 < self.steps.len() {
                    self.current_step += 1;
                    self.step_state = ActionState::Init;
                    ActionState::Executing
                } else {
                    ActionState::Success
                }
            }
        }
    }
    
    fn on_start(&mut self, context: &mut ActionContext) -> bool {
        if self.steps.is_empty() {
            return false;
        }
        
        // Only try to start the first step
        self.steps[0].on_start(context)
    }
    
    fn on_cancel(&mut self, context: &mut ActionContext) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].on_cancel(context);
        }
    }
    
    fn on_success(&mut self, context: &mut ActionContext) {
        // Sequence succeeded, all steps completed
        for step in &mut self.steps {
            step.on_success(context);
        }
    }
    
    fn on_failure(&mut self, context: &mut ActionContext) {
        // Sequence failed at the current step
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].on_failure(context);
        }
    }
    
    fn label(&self) -> &str {
        "Sequence"
    }
}

// Common action implementations

pub struct MoveToTarget {
    speed: f32,
}

impl MoveToTarget {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}

impl Action for MoveToTarget {
    fn execute(&mut self, context: &mut ActionContext) -> ActionState {
        // Get target position
        let target_pos = if let Some(target) = context.target {
            if let Some(tracked) = context.entity_tracker.get_tracked_entity(target) {
                tracked.last_seen_position
            } else {
                return ActionState::Failure;
            }
        } else {
            return ActionState::Failure;
        };
        
        // Move toward target
        if context.executor.move_toward(target_pos, self.speed) {
            ActionState::Success
        } else {
            ActionState::Executing
        }
    }
    
    fn on_start(&mut self, context: &mut ActionContext) -> bool {
        // Verify we have a valid target before starting
        context.target.is_some() && context.entity_tracker.get_tracked_entity(context.target.unwrap()).is_some()
    }
    
    fn label(&self) -> &str {
        "Move To Target"
    }
}

pub struct FleeFromThreat;

impl Action for FleeFromThreat {
    fn execute(&mut self, context: &mut ActionContext) -> ActionState {
        if let Some((_, threat_pos, _)) = context.perception.closest_entity() {
            if context.executor.flee_from(threat_pos) {
                ActionState::Success
            } else {
                ActionState::Executing
            }
        } else {
            ActionState::Success
        }
    }
    
    fn on_start(&mut self, context: &mut ActionContext) -> bool {
        // Only start if there's an actual threat
        context.perception.closest_entity().is_some()
    }
    
    fn label(&self) -> &str {
        "Flee From Threat"
    }
}

pub struct AttackTarget;

impl Action for AttackTarget {
    fn execute(&mut self, context: &mut ActionContext) -> ActionState {
        if context.executor.attack(context.target) {
            ActionState::Success
        } else {
            ActionState::Executing
        }
    }
    
    fn on_start(&mut self, context: &mut ActionContext) -> bool {
        // Verify we have a valid target before starting
        context.target.is_some()
    }
    
    fn label(&self) -> &str {
        "Attack Target"
    }
}

pub struct Rest {
    duration: f32,
    elapsed: f32,
}

impl Rest {
    pub fn new(duration: f32) -> Self {
        Self { duration, elapsed: 0.0 }
    }
}

impl Action for Rest {
    fn execute(&mut self, context: &mut ActionContext) -> ActionState {
        self.elapsed += context.delta_time;
        
        if self.elapsed >= self.duration {
            ActionState::Success
        } else {
            context.executor.idle(self.duration - self.elapsed);
            ActionState::Executing
        }
    }
    
    fn on_start(&mut self, _context: &mut ActionContext) -> bool {
        self.elapsed = 0.0;
        true
    }
    
    fn label(&self) -> &str {
        "Rest"
    }
}