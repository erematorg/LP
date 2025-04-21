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

/// Composite scoring strategies
#[derive(Debug, Clone, Copy)]
pub enum CompositeMode { AllOrNothing, Sum, Product, Max }

/// Combines multiple scorers using a specified strategy
pub struct CompositeScorer {
    scorers: Vec<Box<dyn Scorer + Send + Sync>>,
    weights: Vec<f32>,
    mode: CompositeMode,
    threshold: f32,
    name: String,
}

impl CompositeScorer {
    pub fn new(mode: CompositeMode) -> Self {
        Self {
            scorers: Vec::new(), weights: Vec::new(), mode, threshold: 0.0,
            name: format!("Composite({})", match mode {
                CompositeMode::AllOrNothing => "AllOrNothing",
                CompositeMode::Sum => "Sum", CompositeMode::Product => "Product",
                CompositeMode::Max => "Max"
            }),
        }
    }
    
    pub fn add(mut self, scorer: Box<dyn Scorer + Send + Sync>) -> Self {
        self.scorers.push(scorer); self.weights.push(1.0); self
    }
    
    pub fn add_weighted(mut self, scorer: Box<dyn Scorer + Send + Sync>, weight: f32) -> Self {
        self.scorers.push(scorer); self.weights.push(weight); self
    }
    
    pub fn threshold(mut self, value: f32) -> Self {
        self.threshold = value.clamp(0.0, 1.0); self
    }
    
    pub fn name(mut self, name: &str) -> Self { self.name = name.to_string(); self }
}

impl Scorer for CompositeScorer {
    fn score(&self, context: &ScorerContext) -> Score {
        if self.scorers.is_empty() { return Score::new(0.0); }
        
        let scores: Vec<f32> = self.scorers.iter()
            .map(|s| s.score(context).value()).collect();
        
        let final_score = match self.mode {
            CompositeMode::AllOrNothing => {
                if scores.iter().all(|&s| s >= self.threshold) {
                    scores.iter().zip(self.weights.iter())
                        .map(|(&s, &w)| s * w).sum()
                } else { 0.0 }
            },
            CompositeMode::Sum => {
                let sum: f32 = scores.iter().zip(self.weights.iter())
                    .map(|(&s, &w)| s * w).sum();
                if sum >= self.threshold { sum } else { 0.0 }
            },
            CompositeMode::Product => {
                let product: f32 = scores.iter().zip(self.weights.iter())
                    .map(|(&s, &w)| s.powf(w)).product();
                if product >= self.threshold { product } else { 0.0 }
            },
            CompositeMode::Max => {
                scores.iter().zip(self.weights.iter())
                    .map(|(&s, &w)| s * w).fold(0.0, f32::max)
            },
        };
        
        Score::new(final_score)
    }
    
    fn label(&self) -> &str { &self.name }
}