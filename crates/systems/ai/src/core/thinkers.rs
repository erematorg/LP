//! Thinkers are the "brain" of an entity. You attach Scorers to it, and the
//! Thinker picks the right Action to run based on the resulting Scores.

use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{
    log::{
        Level,
        tracing::{Span, field, span},
    },
    prelude::*,
};

use crate::core::{
    actions::{self, ActionBuilder, ActionBuilderWrapper, ActionState},
    choices::{Choice, ChoiceBuilder},
    pickers::Picker,
    scorers::{Score, ScorerBuilder},
};

/// Wrapper for Actor entities. In terms of Scorers, Thinkers, and Actions,
/// this is the Entity actually performing the action.
#[derive(Debug, Clone, Component, Copy, Reflect)]
pub struct Actor(pub Entity);

#[derive(Debug, Clone, Copy, Reflect)]
pub struct Action(pub Entity);

impl Action {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone, Component)]
pub struct ActionSpan {
    pub(crate) span: Span,
}

impl ActionSpan {
    pub(crate) fn new(action: Entity, label: Option<&str>) -> Self {
        let span = span!(
            Level::DEBUG,
            "action",
            ent = ?action,
            label = field::Empty,
        );
        if let Some(label) = label {
            span.record("label", label);
        }
        Self { span }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct Scorer(pub Entity);

#[derive(Debug, Clone, Component)]
pub struct ScorerSpan {
    pub(crate) span: Span,
}

impl ScorerSpan {
    pub(crate) fn new(scorer: Entity, label: Option<&str>) -> Self {
        let span = span!(
            Level::DEBUG,
            "scorer",
            ent = ?scorer,
            label = field::Empty,
        );

        if let Some(label) = label {
            span.record("label", label);
        }
        Self { span }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// The "brains" behind this whole operation. A `Thinker` is what glues
/// together `Actions` and `Scorers` and shapes larger, intelligent-seeming
/// systems.
#[derive(Component, Debug, Reflect)]
#[reflect(from_reflect = false)]
pub struct Thinker {
    #[reflect(ignore)]
    picker: Arc<dyn Picker>,
    #[reflect(ignore)]
    otherwise: Option<ActionBuilderWrapper>,
    #[reflect(ignore)]
    choices: Vec<Choice>,
    #[reflect(ignore)]
    current_action: Option<(Action, ActionBuilderWrapper)>,
    current_action_label: Option<Option<String>>,
    #[reflect(ignore)]
    span: Span,
    #[reflect(ignore)]
    scheduled_actions: VecDeque<ActionBuilderWrapper>,
}

impl Thinker {
    /// Make a new [`ThinkerBuilder`].
    pub fn build() -> ThinkerBuilder {
        ThinkerBuilder::new()
    }

    pub fn schedule_action(&mut self, action: impl ActionBuilder + 'static) {
        self.scheduled_actions
            .push_back(ActionBuilderWrapper::new(Arc::new(action)));
    }
}

/// This is what you actually use to configure Thinker behavior.
#[derive(Component, Clone, Debug, Default)]
pub struct ThinkerBuilder {
    picker: Option<Arc<dyn Picker>>,
    otherwise: Option<ActionBuilderWrapper>,
    choices: Vec<ChoiceBuilder>,
    label: Option<String>,
}

impl ThinkerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            picker: None,
            otherwise: None,
            choices: Vec::new(),
            label: None,
        }
    }

    /// Define a [`Picker`] for this Thinker.
    pub fn picker(mut self, picker: impl Picker + 'static) -> Self {
        self.picker = Some(Arc::new(picker));
        self
    }

    /// Define an [`ActionBuilder`] and [`ScorerBuilder`] pair.
    pub fn when(
        mut self,
        scorer: impl ScorerBuilder + 'static,
        action: impl ActionBuilder + 'static,
    ) -> Self {
        self.choices
            .push(ChoiceBuilder::new(Arc::new(scorer), Arc::new(action)));
        self
    }

    /// Default `Action` to execute if the `Picker` did not pick any choices.
    pub fn otherwise(mut self, otherwise: impl ActionBuilder + 'static) -> Self {
        self.otherwise = Some(ActionBuilderWrapper::new(Arc::new(otherwise)));
        self
    }

    /// Configures a label to use for the thinker when logging.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().to_string());
        self
    }
}

impl ActionBuilder for ThinkerBuilder {
    fn build(&self, cmd: &mut Commands, action_ent: Entity, actor: Entity) {
        let span = span!(
            Level::DEBUG,
            "thinker",
            actor = ?actor,
        );
        let _guard = span.enter();
        debug!("Spawning Thinker.");
        let choices = self
            .choices
            .iter()
            .map(|choice| choice.build(cmd, actor, action_ent))
            .collect();
        std::mem::drop(_guard);
        cmd.entity(action_ent)
            .insert(Thinker {
                picker: self
                    .picker
                    .clone()
                    .expect("ThinkerBuilder must have a Picker"),
                otherwise: self.otherwise.clone(),
                choices,
                current_action: None,
                current_action_label: None,
                span,
                scheduled_actions: VecDeque::new(),
            })
            .insert(Name::new("Thinker"))
            .insert(ActionState::Requested);
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

pub fn thinker_component_attach_system(
    mut cmd: Commands,
    q: Query<(Entity, &ThinkerBuilder), Without<HasThinker>>,
) {
    for (entity, thinker_builder) in q.iter() {
        let thinker = actions::spawn_action(thinker_builder, &mut cmd, entity);
        cmd.entity(entity).insert(HasThinker(thinker));
    }
}

pub fn thinker_component_detach_system(
    mut cmd: Commands,
    q: Query<(Entity, &HasThinker), Without<ThinkerBuilder>>,
) {
    for (actor, HasThinker(thinker)) in q.iter() {
        if let Ok(mut ent) = cmd.get_entity(*thinker) {
            ent.despawn();
        }
        cmd.entity(actor).remove::<HasThinker>();
    }
}

pub fn actor_gone_cleanup(
    mut cmd: Commands,
    actors: Query<&ThinkerBuilder>,
    q: Query<(Entity, &Actor)>,
) {
    for (child, Actor(actor)) in q.iter() {
        if actors.get(*actor).is_err() {
            if let Ok(mut ent) = cmd.get_entity(child) {
                ent.despawn();
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct HasThinker(Entity);

impl HasThinker {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub struct ThinkerIterations {
    index: usize,
    max_duration: Duration,
}
impl ThinkerIterations {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            index: 0,
            max_duration,
        }
    }
}
impl Default for ThinkerIterations {
    fn default() -> Self {
        Self::new(Duration::from_millis(10))
    }
}

pub fn thinker_system(
    mut cmd: Commands,
    mut iterations: Local<ThinkerIterations>,
    mut thinker_q: Query<(Entity, &Actor, &mut Thinker)>,
    scores: Query<&Score>,
    mut action_states: Query<&mut actions::ActionState>,
    action_spans: Query<&ActionSpan>,
    scorer_spans: Query<&ScorerSpan>,
) {
    let start = Instant::now();
    for (thinker_ent, Actor(actor), mut thinker) in thinker_q.iter_mut().skip(iterations.index) {
        iterations.index += 1;

        let thinker_state = action_states
            .get_mut(thinker_ent)
            .expect("Where is it?")
            .clone();

        let thinker_span = thinker.span.clone();
        let _thinker_span_guard = thinker_span.enter();

        match thinker_state {
            ActionState::Init => {
                let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                debug!("Initializing thinker.");
                *act_state = ActionState::Requested;
            }
            ActionState::Requested => {
                let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                debug!("Starting execution.");
                *act_state = ActionState::Executing;
            }
            ActionState::Success | ActionState::Failure => {}
            ActionState::Cancelled => {
                debug!("Cleaning up.");
                if let Some(current) = &mut thinker.current_action {
                    let action_span = action_spans.get(current.0.0).expect("Where is it?");
                    debug!("Cancelling current action.");
                    let state = action_states
                        .get_mut(current.0.0)
                        .expect("Missing current action")
                        .clone();
                    match state {
                        ActionState::Success | ActionState::Failure => {
                            debug!("Action already wrapped up.");
                            if let Ok(mut ent) = cmd.get_entity(current.0.0) {
                                ent.despawn();
                            }
                            thinker.current_action = None;
                        }
                        ActionState::Cancelled => {
                            debug!("Already cancelled.");
                        }
                        _ => {
                            let mut state =
                                action_states.get_mut(current.0.0).expect("Missing action");
                            debug!("Action still executing. Cancelling it.");
                            action_span.span.in_scope(|| {
                                debug!("Cancelling action.");
                            });
                            *state = ActionState::Cancelled;
                        }
                    }
                } else {
                    let mut act_state = action_states.get_mut(thinker_ent).expect("???");
                    debug!("No current action. Completing as Success.");
                    *act_state = ActionState::Success;
                }
            }
            ActionState::Executing => {
                #[cfg(feature = "trace")]
                trace!("Thinker is executing. Thinking...");
                if let Some(choice) = thinker.picker.pick(&thinker.choices, &scores) {
                    #[cfg(feature = "trace")]
                    trace!("Action picked. Executing picked action.");
                    let action = choice.action.clone();
                    let scorer = choice.scorer;
                    let score = scores.get(choice.scorer.0).expect("Where is it?");
                    exec_picked_action(
                        &mut cmd,
                        *actor,
                        &mut thinker,
                        &action,
                        &mut action_states,
                        &action_spans,
                        Some((&scorer, score)),
                        &scorer_spans,
                        true,
                    );
                } else if should_schedule_action(&mut thinker, &mut action_states) {
                    debug!("Spawning scheduled action.");
                    let action = thinker
                        .scheduled_actions
                        .pop_front()
                        .expect("we literally just checked if it was there.");
                    let new_action = actions::spawn_action(action.1.as_ref(), &mut cmd, *actor);
                    thinker.current_action = Some((Action(new_action), action.clone()));
                    thinker.current_action_label = Some(action.1.label().map(|s| s.into()));
                } else if let Some(default_action_ent) = &thinker.otherwise {
                    let default_action_ent = default_action_ent.clone();
                    exec_picked_action(
                        &mut cmd,
                        *actor,
                        &mut thinker,
                        &default_action_ent,
                        &mut action_states,
                        &action_spans,
                        None,
                        &scorer_spans,
                        false,
                    );
                } else if let Some((action_ent, _)) = &thinker.current_action {
                    let action_span = action_spans.get(action_ent.0).expect("Where is it?");
                    let _guard = action_span.span.enter();
                    let mut curr_action_state = action_states
                        .get_mut(action_ent.0)
                        .expect("Missing current action");
                    let previous_done = matches!(
                        *curr_action_state,
                        ActionState::Success | ActionState::Failure
                    );
                    if previous_done {
                        debug!("Action completed. Despawning.");
                        if let Ok(mut ent) = cmd.get_entity(action_ent.0) {
                            ent.despawn();
                        }
                        thinker.current_action = None;
                    } else if *curr_action_state == ActionState::Init {
                        *curr_action_state = ActionState::Requested;
                    }
                }
            }
        }
        if iterations.index % 500 == 0 && start.elapsed() > iterations.max_duration {
            return;
        }
    }
    iterations.index = 0;
}

fn should_schedule_action(
    thinker: &mut Mut<Thinker>,
    states: &mut Query<&mut ActionState>,
) -> bool {
    #[cfg(feature = "trace")]
    let thinker_span = thinker.span.clone();
    #[cfg(feature = "trace")]
    let _thinker_span_guard = thinker_span.enter();
    if thinker.scheduled_actions.is_empty() {
        #[cfg(feature = "trace")]
        trace!("No scheduled actions. Not scheduling anything.");
        false
    } else if let Some((action_ent, _)) = &mut thinker.current_action {
        let curr_action_state = states
            .get_mut(action_ent.0)
            .expect("Missing current action");

        let action_done = matches!(
            *curr_action_state,
            ActionState::Success | ActionState::Failure
        );

        #[cfg(feature = "trace")]
        if action_done {
            trace!("Current action is already done. Can schedule.");
        } else {
            trace!("Current action is still executing. Not scheduling anything.");
        }

        action_done
    } else {
        #[cfg(feature = "trace")]
        trace!("No current action actions. Can schedule.");
        true
    }
}

#[allow(clippy::too_many_arguments)]
fn exec_picked_action(
    cmd: &mut Commands,
    actor: Entity,
    thinker: &mut Mut<Thinker>,
    picked_action: &ActionBuilderWrapper,
    states: &mut Query<&mut ActionState>,
    action_spans: &Query<&ActionSpan>,
    scorer_info: Option<(&Scorer, &Score)>,
    scorer_spans: &Query<&ScorerSpan>,
    override_current: bool,
) {
    let thinker_span = thinker.span.clone();
    let _thinker_span_guard = thinker_span.enter();
    if let Some((action_ent, ActionBuilderWrapper(current_id, _))) = &mut thinker.current_action {
        let mut curr_action_state = states
            .get_mut(action_ent.0)
            .expect("Missing current action");
        let previous_done = matches!(
            *curr_action_state,
            ActionState::Success | ActionState::Failure
        );
        let action_span = action_spans.get(action_ent.0).expect("Where is it?");
        let _guard = action_span.span.enter();
        if (!Arc::ptr_eq(current_id, &picked_action.0) && override_current) || previous_done {
            if !previous_done {
                if override_current {
                    #[cfg(feature = "trace")]
                    trace!("Falling back to `otherwise` clause.",);
                } else {
                    #[cfg(feature = "trace")]
                    trace!("Picked a different action than the current one.",);
                }
            }
            match *curr_action_state {
                ActionState::Executing | ActionState::Requested => {
                    debug!("Requesting cancellation.");
                    *curr_action_state = ActionState::Cancelled;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    debug!("Previous action completed. Despawning.");
                    if let Ok(mut ent) = cmd.get_entity(action_ent.0) {
                        ent.despawn();
                    }
                    if let Some((Scorer(ent), score)) = scorer_info {
                        let scorer_span = scorer_spans.get(*ent).expect("Where is it?");
                        let _guard = scorer_span.span.enter();
                        debug!("Winning score: {}", score.get());
                    }
                    std::mem::drop(_guard);
                    debug!("Spawning next action");
                    let new_action =
                        Action(actions::spawn_action(picked_action.1.as_ref(), cmd, actor));
                    thinker.current_action = Some((new_action, picked_action.clone()));
                    thinker.current_action_label = Some(picked_action.1.label().map(|s| s.into()));
                }
                ActionState::Cancelled => {
                    #[cfg(feature = "trace")]
                    trace!("Cancellation already requested. Waiting.");
                }
            };
        } else if *curr_action_state == ActionState::Init {
            *curr_action_state = ActionState::Requested;
        }
        #[cfg(feature = "trace")]
        trace!("Continuing execution of current action.",)
    } else {
        #[cfg(feature = "trace")]
        trace!("Falling back to `otherwise` clause.",);

        if let Some((Scorer(ent), score)) = scorer_info {
            let scorer_span = scorer_spans.get(*ent).expect("Where is it?");
            let _guard = scorer_span.span.enter();
            debug!("Winning score: {}", score.get());
        }
        debug!("No current action. Spawning new.");
        let new_action = actions::spawn_action(picked_action.1.as_ref(), cmd, actor);
        thinker.current_action = Some((Action(new_action), picked_action.clone()));
        thinker.current_action_label = Some(picked_action.1.label().map(|s| s.into()));
    }
}
