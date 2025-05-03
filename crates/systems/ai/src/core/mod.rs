pub mod actions;
pub mod evaluators;
pub mod scorers;
pub mod thinkers;
pub mod utility;
/// Prelude for the core AI module.
/// 
/// This includes the most common types in the core module,
/// re-exported for convenience.
pub mod prelude {
    pub use crate::core::utility::{Behavior, UtilityScore, determine_behavior, 
                                 UtilityCache, EntityUtilityCache, CacheableModule};
    pub use crate::core::actions::{Action, ActionContext, ActionState, 
                                 Sequence, Concurrent, ConcurrentMode};
    pub use crate::core::scorers::{Score, Scorer, ScorerContext, 
                                 CompositeScorer, CompositeMode};
    pub use crate::core::thinkers::{Thinker, Choice, ActionType, 
                                  Picker, FirstToScore, Highest, HasThinker};
}