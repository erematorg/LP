use crate::prelude::*;
use bevy::prelude::*;

/// Component representing a score between 0.0 and 1.0
#[derive(Debug, Clone, Copy)]
pub struct Score(pub f32);

impl Score {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
    
    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Trait for components that evaluate world state and produce scores
pub trait Scorer: Send + Sync {
    /// Calculate a score based on current context
    fn score(&self, context: &ScorerContext) -> Score;
    
    /// Label for debugging and tracing
    fn label(&self) -> &str {
        "Unnamed Scorer" 
    }
}

/// Context provided to scorers for evaluation
pub struct ScorerContext<'a> {
    pub perception: &'a Perception,
    pub entity_tracker: &'a EntityTracker,
    pub needs_tracker: &'a NeedsTracker,
    pub personality: Option<&'a Personality>,
    pub social_network: Option<&'a SocialNetwork>,
    pub current_position: Vec2,
}

// Move these mapping functions from controller.rs
pub fn map_perception_to_behavior(perception: &Perception) -> Behavior {
    if perception.highest_threat_level > 0.7 { Behavior::Flee }
    else if perception.highest_threat_level > 0.4 { Behavior::Fight }
    else { Behavior::Explore }
}

pub fn map_entity_tracker_to_behavior(tracker: &EntityTracker) -> Behavior {
    match tracker.get_most_important_entity() {
        Some((_, entity)) if entity.importance > 0.7 => Behavior::Hunt,
        Some(_) => Behavior::Explore,
        None => Behavior::Idle
    }
}

pub fn map_needs_to_behavior(needs: &NeedsTracker) -> Behavior {
    match needs.get_most_urgent_need() {
        Some((NeedType::Hunger, _)) => Behavior::Hunt,
        Some((NeedType::Safety, _)) => Behavior::Flee,
        Some((NeedType::Rest, _)) => Behavior::Rest,
        Some((NeedType::Social, _)) => Behavior::Socialize,
        None => Behavior::Idle
    }
}

// Basic scorer implementations
pub struct PerceptionScorer;
impl Scorer for PerceptionScorer {
    fn score(&self, context: &ScorerContext) -> Score {
        Score::new(context.perception.highest_threat_level)
    }
    
    fn label(&self) -> &str {
        "Perception"
    }
}

pub struct NeedScorer {
    pub need_type: NeedType,
}

impl Scorer for NeedScorer {
    fn score(&self, context: &ScorerContext) -> Score {
        if let Some((need_type, urgency)) = context.needs_tracker.get_most_urgent_need() {
            if need_type == self.need_type {
                return Score::new(urgency.value());
            }
        }
        Score::new(0.0)
    }
    
    fn label(&self) -> &str {
        match self.need_type {
            NeedType::Hunger => "Hunger Need",
            NeedType::Safety => "Safety Need",
            NeedType::Rest => "Rest Need",
            NeedType::Social => "Social Need",
        }
    }
}