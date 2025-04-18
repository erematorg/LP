// ai/src/core/thinkers.rs

use crate::prelude::*;
use bevy::prelude::*;

use std::collections::VecDeque;

/// A choice representing a potential action with its scorer
pub struct Choice {
    pub scorer: Box<dyn Scorer + Send + Sync>,
    pub action_type: ActionType,
}

/// Enum representing different action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    MoveToTarget,
    FleeFromThreat,
    AttackTarget,
    Rest,
    Idle,
}

impl ActionType {
    pub fn create(&self) -> Box<dyn Action + Send + Sync> {
        match self {
            ActionType::MoveToTarget => Box::new(super::actions::MoveToTarget::new(1.0)),
            ActionType::FleeFromThreat => Box::new(super::actions::FleeFromThreat),
            ActionType::AttackTarget => Box::new(super::actions::AttackTarget),
            ActionType::Rest => Box::new(super::actions::Rest::new(1.0)),
            ActionType::Idle => Box::new(super::actions::Rest::new(0.5)),
        }
    }
    
    pub fn to_behavior(&self) -> Behavior {
        match self {
            ActionType::MoveToTarget => Behavior::Hunt,
            ActionType::FleeFromThreat => Behavior::Flee,
            ActionType::AttackTarget => Behavior::Fight,
            ActionType::Rest => Behavior::Rest,
            ActionType::Idle => Behavior::Idle,
        }
    }
}

/// Strategy for picking the best action based on scores
pub trait Picker: Send + Sync {
    fn pick<'a>(&self, choices: &'a [(&'a Choice, Score)]) -> Option<&'a Choice>;
}

/// Picks the first choice with a score above the threshold
pub struct FirstToScore {
    pub threshold: f32,
}

impl Picker for FirstToScore {
    fn pick<'a>(&self, choices: &'a [(&'a Choice, Score)]) -> Option<&'a Choice> {
        for (choice, score) in choices {
            if score.value() >= self.threshold {
                return Some(choice);
            }
        }
        None
    }
}

/// Picks the choice with the highest score
pub struct Highest;

impl Picker for Highest {
    fn pick<'a>(&self, choices: &'a [(&'a Choice, Score)]) -> Option<&'a Choice> {
        choices.iter()
            .max_by(|(_, a_score), (_, b_score)| 
                a_score.value().partial_cmp(&b_score.value()).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(choice, _)| *choice)
    }
}

/// Main decision-making component
pub struct Thinker {
    pub choices: Vec<Choice>,
    pub picker: Box<dyn Picker + Send + Sync>,
    pub current_action: Option<(Box<dyn Action + Send + Sync>, ActionState)>,
    pub otherwise_action: Option<ActionType>,
    pub scheduled_actions: VecDeque<ActionType>,
}

impl Thinker {
    pub fn new(picker: Box<dyn Picker + Send + Sync>) -> Self {
        Self {
            choices: Vec::new(),
            picker,
            current_action: None,
            otherwise_action: None,
            scheduled_actions: VecDeque::new(),
        }
    }
    
    pub fn when(&mut self, scorer: Box<dyn Scorer + Send + Sync>, action_type: ActionType) {
        self.choices.push(Choice { 
            scorer, 
            action_type
        });
    }
    
    pub fn otherwise(&mut self, action_type: ActionType) {
        self.otherwise_action = Some(action_type);
    }
    
    pub fn schedule_action(&mut self, action_type: ActionType) {
        self.scheduled_actions.push_back(action_type);
    }
    
    pub fn update(&mut self, context: &ScorerContext, action_context: &mut ActionContext) -> Behavior {
        // Continue current action if it exists
        if let Some((action, state)) = &mut self.current_action {
            match *state {
                ActionState::Executing | ActionState::Requested => {
                    *state = action.execute(action_context);
                },
                ActionState::Cancelled => {
                    action.on_cancel(action_context);
                    *state = ActionState::Failure;
                },
                ActionState::Success | ActionState::Failure => {
                    self.current_action = None;
                },
                ActionState::Init => {
                    *state = ActionState::Requested;
                }
            }
        }
        
        // Select new action if needed
        if self.current_action.is_none() {
            if let Some(action_type) = self.get_next_action_type(context) {
                self.current_action = Some((action_type.create(), ActionState::Init));
                return action_type.to_behavior();
            }
        }
        
        // Determine the current behavior type
        self.current_action.as_ref()
            .map(|(_, state)| {
                if *state == ActionState::Success || *state == ActionState::Failure {
                    Behavior::Idle
                } else {
                    match self.scheduled_actions.front() {
                        Some(action_type) => action_type.to_behavior(),
                        None => Behavior::Idle
                    }
                }
            })
            .unwrap_or(Behavior::Idle)
    }
    
    fn get_next_action_type(&mut self, context: &ScorerContext) -> Option<ActionType> {
        // Check scheduled actions first
        if let Some(action_type) = self.scheduled_actions.pop_front() {
            return Some(action_type);
        }
        
        // Score all choices
        let scored_choices: Vec<_> = self.choices.iter()
            .map(|choice| (choice, choice.scorer.score(context)))
            .collect();
        
        // Use picker to select best choice
        if let Some(choice) = self.picker.pick(&scored_choices) {
            return Some(choice.action_type);
        }
        
        // Otherwise action
        self.otherwise_action
    }
}

/// Component to attach thinker to an entity
#[derive(Component)]
pub struct HasThinker {
    pub thinker: Thinker,
}