# LP AI Architecture

This crate provides the agent decision loop used across Life's Progress. Every organism—plant, fungus, animal, machine—shares the same ECS-driven pipeline:

```
Perception → Trackers & Drives → State Layers (memory/emotion/social) → Utility Arbiter → Behavior Executors
```

## Key Concepts

### AIModule
Any component implementing `AIModule` exposes `update()` and `utility()` (0-1). Existing modules:
- **Perception** - sensory cache (vision/chemical/etc.)
- **Threat / Prey trackers** - context-specific evaluation
- **Needs tracker** - energy/homeostasis drives
- **Memory events** - short-term context
- **Personality / Social graphs** - trait-based biasing

Modules can publish `IntentContribution` events to compete for control (see Arbiter below).

### Utility Arbiter
`arbiter::UtilityArbiterPlugin` resets each agent’s `IntentSelection` every frame, reads `IntentContribution` events, applies continuation bias/hysteresis, and records the winning module & score. Only one intent wins per frame, providing Rain-World-style “strongest drive wins” behavior.

### Behavior Executors
Systems downstream of the arbiter read `IntentSelection` and perform actions (movement, growth, interaction). Executors will be built per intent (e.g., flee, forage, rest) and consume data from trackers/drives to apply forces or state changes.

## Adding a New Decision Module
1. Create a component implementing `AIModule` (e.g., `#[derive(Component)] struct HungerTracker`).
2. In your system, update the module’s internal state and call `IntentContribution` to report a utility:
```rust
fn hunger_intent_system(
    query: Query<(Entity, &HungerTracker)>,
    mut writer: MessageWriter<IntentContribution>,
) {
    for (entity, tracker) in &query {
        writer.write(IntentContribution {
            entity,
            module: "hunger",
            utility: tracker.utility(),
        });
    }
}
```
3. The arbiter will compare utilities (with continuation bias if the same module won last frame) and set `IntentSelection`.

## Behavior Executors (WIP)
Executors subscribe to `IntentSelection`:
```rust
fn flee_executor(
    query: Query<(Entity, &IntentSelection, &mut Velocity), With<ThreatTracker>>,
) {
    // if `selection.winner == Some("flee")`, apply movement logic
}
```

### End-to-end example
See `examples/basic_ai.rs` for a minimal wiring of the full loop:
1. Each creature owns `Perception`, `EntityTracker`, `NeedsTracker`, `ThreatTracker`, `PreyTracker`, and `IntentSelection`.
2. Sensors fill the `EntityTracker`; tracker systems evaluate utilities and emit `IntentContribution` events via the arbiter.
3. A behavior executor reads `IntentResolved` events, maps winners (`"threat"`, `"prey"`, `"needs"`) to locomotion commands, and world systems react to the actions.

## Philosophy
- **No DSL / behavior trees** - everything is ECS data.
- **Species-agnostic** - modules work for plants/animals alike; species differences are just component presets.
- **Emergence-first** - utilities are derived from physics-aware trackers/drives, not hand-authored scripts.
