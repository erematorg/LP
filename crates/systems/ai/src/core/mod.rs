//! Core AI module for LP
//! 
//! Provides a utility AI system with composable scorers, actions, and thinkers.
//! Based on the Big Brain architecture but integrated into LP's ecosystem.

pub mod actions;
pub mod choices;
pub mod evaluators;
pub mod measures;
pub mod pickers;
pub mod scorers;
pub mod thinkers;
pub mod utility;

/// Prelude for the core AI module.
/// 
/// This includes the most common types you'll need when working with LP's AI system.
/// Use with `use crate::systems::ai::core::prelude::*;`
pub mod prelude {
    // Actions (ActionBuilder and ActionState are in actions, but Action is in thinkers)
    pub use crate::core::actions::{
        ActionBuilder, ActionState, ConcurrentMode, Concurrently, Steps
    };
    
    // Scorers  
    pub use crate::core::scorers::{
        AllOrNothing, EvaluatingScorer, FixedScore, MeasuredScorer, ProductOfScorers, 
        Score, ScorerBuilder, SumOfScorers, WinningScorer
    };
    
    // Thinkers (includes Action, Actor, etc.)
    pub use crate::core::thinkers::{
        Action, ActionSpan, Actor, HasThinker, Scorer, ScorerSpan, Thinker, ThinkerBuilder
    };
    
    // Evaluators
    pub use crate::core::evaluators::{
        Evaluator, LinearEvaluator, PowerEvaluator, SigmoidEvaluator
    };
    
    // Measures
    pub use crate::core::measures::{
        ChebyshevDistance, Measure, WeightedProduct, WeightedSum
    };
    
    // Pickers
    pub use crate::core::pickers::{
        FirstToScore, Highest, HighestToScore, Picker
    };
    
    // Choices
    pub use crate::core::choices::{Choice, ChoiceBuilder};

    pub use crate::core::utility::*;
}

use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

/// Core plugin for LP's AI system. Provides utility AI functionality.
/// 
/// Add this plugin to enable AI behaviors in your app.
/// 
/// # Example
/// ```rust
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(LPAIPlugin::new(PreUpdate))
///     .run();
/// ```
#[derive(Debug, Clone, Reflect)]
#[reflect(from_reflect = false)]
pub struct LPAIPlugin {
    #[reflect(ignore)]
    schedule: Interned<dyn ScheduleLabel>,
    #[reflect(ignore)]
    cleanup_schedule: Interned<dyn ScheduleLabel>,
}

impl LPAIPlugin {
    /// Create the AI plugin which runs in the specified schedule
    pub fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
            cleanup_schedule: Last.intern(),
        }
    }
    
    /// Set the schedule for cleanup tasks (default: Last)
    pub fn with_cleanup_schedule(mut self, cleanup_schedule: impl ScheduleLabel) -> Self {
        self.cleanup_schedule = cleanup_schedule.intern();
        self
    }
}

impl Plugin for LPAIPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            self.schedule.intern(),
            (
                AISet::Scorers,
                AISet::Thinkers,
                AISet::Actions,
            ).chain(),
        )
        .configure_sets(self.cleanup_schedule.intern(), AISet::Cleanup)
        
        // Add scorer systems
        .add_systems(
            self.schedule.intern(),
            (
                scorers::fixed_score_system,
                scorers::measured_scorers_system,
                scorers::all_or_nothing_system,
                scorers::sum_of_scorers_system,
                scorers::product_of_scorers_system,
                scorers::winning_scorer_system,
                scorers::evaluating_scorer_system,
            ).in_set(AISet::Scorers),
        )
        
        // Add thinker systems
        .add_systems(
            self.schedule.intern(),
            thinkers::thinker_system.in_set(AISet::Thinkers),
        )
        
        // Add action systems
        .add_systems(
            self.schedule.intern(),
            (actions::steps_system, actions::concurrent_system).in_set(AISet::Actions),
        )
        
        // Add cleanup systems
        .add_systems(
            self.cleanup_schedule.intern(),
            (
                thinkers::thinker_component_attach_system,
                thinkers::thinker_component_detach_system,
                thinkers::actor_gone_cleanup,
            ).in_set(AISet::Cleanup),
        );
    }
}

/// System sets for organizing AI-related systems
#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet, Reflect)]
pub enum AISet {
    /// Scorers evaluate world state
    Scorers,
    /// Thinkers make decisions based on scores
    Thinkers,
    /// Actions execute the chosen behaviors
    Actions,
    /// Cleanup tasks run last
    Cleanup,
}