use std::collections::HashMap;

use bevy::prelude::*;

use crate::AIModule;
use crate::trackers::needs_tracker::NeedsTracker;
use crate::trackers::prey_tracker::PreyTracker;
use crate::trackers::threat_tracker::ThreatTracker;

/// Schedule sets that ensure the arbiter runs in a predictable order each frame.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ArbiterSet {
    Reset,
    Gather,
    Evaluate,
    Broadcast,
}

/// Component describing the intent chosen for an agent this frame.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct IntentSelection {
    /// Winner chosen this frame.
    pub winner: Option<&'static str>,
    /// Utility returned by the winning module (biased).
    pub utility: f32,
    /// Winner from the previous frame (used for continuation bias).
    pub last_winner: Option<&'static str>,
}

impl IntentSelection {
    fn reset_for_frame(&mut self) {
        self.last_winner = self.winner;
        self.winner = None;
        self.utility = 0.0;
    }
}

/// Resource storing arbiter tuning parameters.
#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct ArbiterConfig {
    /// How much advantage we give to the previous winner when scores are similar.
    pub continuation_bias: f32,
}

impl Default for ArbiterConfig {
    fn default() -> Self {
        Self {
            continuation_bias: 0.05,
        }
    }
}

/// Event emitted by modules competing for control.
#[derive(Message, Clone, Debug)]
pub struct IntentContribution {
    pub entity: Entity,
    pub module: &'static str,
    pub utility: f32,
}

/// Event emitted after the arbiter resolves the winning intent.
#[derive(Message, Clone, Debug)]
pub struct IntentResolved {
    pub entity: Entity,
    pub winner: Option<&'static str>,
    pub utility: f32,
}

fn reset_intentions(mut query: Query<&mut IntentSelection>) {
    for mut selection in &mut query {
        selection.reset_for_frame();
    }
}

fn gather_need_intents(
    query: Query<(Entity, &NeedsTracker)>,
    mut writer: MessageWriter<IntentContribution>,
) {
    for (entity, tracker) in &query {
        let utility = tracker.utility();
        if utility > 0.0 {
            writer.write(IntentContribution {
                entity,
                module: "needs",
                utility,
            });
        }
    }
}

fn gather_threat_intents(
    query: Query<(Entity, &ThreatTracker)>,
    mut writer: MessageWriter<IntentContribution>,
) {
    for (entity, tracker) in &query {
        let utility = tracker.utility();
        if utility > 0.0 {
            writer.write(IntentContribution {
                entity,
                module: "threat",
                utility,
            });
        }
    }
}

fn gather_prey_intents(
    query: Query<(Entity, &PreyTracker)>,
    mut writer: MessageWriter<IntentContribution>,
) {
    for (entity, tracker) in &query {
        let utility = tracker.utility();
        if utility > 0.0 {
            writer.write(IntentContribution {
                entity,
                module: "prey",
                utility,
            });
        }
    }
}

fn evaluate_intentions(
    config: Res<ArbiterConfig>,
    mut contributions: MessageReader<IntentContribution>,
    mut selections: Query<&mut IntentSelection>,
) {
    // Track the best score per entity within this frame.
    let mut best_scores: HashMap<Entity, f32> = HashMap::default();

    for contribution in contributions.read() {
        if let Ok(mut selection) = selections.get_mut(contribution.entity) {
            let bias = if selection.last_winner == Some(contribution.module) {
                config.continuation_bias
            } else {
                0.0
            };

            let adjusted = (contribution.utility + bias).clamp(0.0, 1.0);
            let current_best = best_scores.entry(contribution.entity).or_insert(0.0);

            if adjusted > *current_best {
                *current_best = adjusted;
                selection.utility = adjusted;
                selection.winner = Some(contribution.module);
            }
        }
    }
}

fn broadcast_intent_selections(
    query: Query<(Entity, &IntentSelection)>,
    mut writer: MessageWriter<IntentResolved>,
) {
    for (entity, selection) in &query {
        writer.write(IntentResolved {
            entity,
            winner: selection.winner,
            utility: selection.utility,
        });
    }
}

/// Plugin wiring the utility arbiter into the Bevy schedule.
#[derive(Default)]
pub struct UtilityArbiterPlugin;

impl Plugin for UtilityArbiterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArbiterConfig>()
            .register_type::<ArbiterConfig>()
            .register_type::<IntentSelection>()
            .add_message::<IntentContribution>()
            .add_message::<IntentResolved>()
            .configure_sets(
                Update,
                (
                    ArbiterSet::Reset,
                    ArbiterSet::Gather,
                    ArbiterSet::Evaluate,
                    ArbiterSet::Broadcast,
                )
                    .chain(),
            )
            .add_systems(Update, reset_intentions.in_set(ArbiterSet::Reset))
            .add_systems(
                Update,
                (
                    gather_need_intents,
                    gather_threat_intents,
                    gather_prey_intents,
                )
                    .in_set(ArbiterSet::Gather),
            )
            .add_systems(Update, evaluate_intentions.in_set(ArbiterSet::Evaluate))
            .add_systems(
                Update,
                broadcast_intent_selections.in_set(ArbiterSet::Broadcast),
            );
    }
}

pub mod prelude {
    pub use super::{
        ArbiterConfig, ArbiterSet, IntentContribution, IntentResolved, IntentSelection,
        UtilityArbiterPlugin,
    };
}
