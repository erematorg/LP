use bevy::prelude::*;
use systems::ai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            LPAIPlugin::default(),
            SocialPlugin,
            DrivesPlugin,
            PersonalityPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(PreyConfig {
            memory_decay_per_second: 1.0,
            max_attractive_distance: 300.0,
            forget_after: 5.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_trackers,
                move_creatures,
                handle_food_consumption,
                (respawn_food, update_visuals, update_labels),
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Creature {
    velocity: Vec2,
    hunger: f32,
    food_consumed: u32,
}

#[derive(Component)]
struct Food {
    respawn_timer: Timer,
    active: bool,
}

#[derive(Component)]
struct CreatureLabel {
    creature: Entity,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn creatures with AI trackers
    for i in 0..8 {
        let x = ((i % 4) as f32 - 1.5) * 80.0;
        let y = (i as f32 / 4.0).floor() * 100.0 - 50.0;

        let entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.9, 0.2),
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                Creature {
                    velocity: Vec2::ZERO,
                    hunger: 0.0,
                    food_consumed: 0,
                },
                // AI components - use refactored tracker system
                EntityTracker::new(20),
                PreyTracker::default(),
                Personality::default(),
            ))
            .id();

        // Label
        commands.spawn((
            Text2d::new("Neutral"),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_translation(Vec3::new(x, y + 25.0, 1.0))
                .with_scale(Vec3::splat(0.5)),
            CreatureLabel { creature: entity },
        ));
    }

    // Spawn food
    for i in 0..16 {
        let x = ((i % 4) as f32 - 1.5) * 120.0;
        let y = ((i / 4) as f32 - 1.5) * 80.0;

        commands.spawn((
            Sprite {
                color: Color::srgb(0.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Food {
                respawn_timer: Timer::from_seconds(10.0, TimerMode::Once),
                active: true,
            },
        ));
    }
}

/// Update entity trackers with visible food
fn update_trackers(
    time: Res<Time>,
    config: Res<PreyConfig>,
    food_query: Query<(Entity, &Transform, &Food)>,
    mut creature_query: Query<(&Transform, &mut EntityTracker, &mut PreyTracker)>,
) {
    let current_time = time.elapsed_secs();

    // Get active food positions
    let active_food: Vec<(Entity, Vec2)> = food_query
        .iter()
        .filter(|(_, _, food)| food.active)
        .map(|(entity, transform, _)| (entity, transform.translation.truncate()))
        .collect();

    for (creature_transform, mut tracker, mut prey_tracker) in &mut creature_query {
        let creature_pos = creature_transform.translation.truncate();

        // Track all visible food (within perception range)
        for (food_entity, food_pos) in &active_food {
            let distance = creature_pos.distance(*food_pos);

            if distance < 300.0 {
                // Attractiveness based on hunger urgency
                tracker.track_entity(
                    *food_entity,
                    *food_pos,
                    distance,
                    current_time,
                    EntityMetadata::Prey {
                        attractiveness: 1.0,
                    },
                );
            }
        }

        // Clean up stale entries
        tracker.forget_old_entities(current_time, config.forget_after);

        // Evaluate tracked food using refactored system
        prey_tracker.update(&tracker, current_time, &config);
    }
}

/// Move creatures toward best food using tracker evaluation
fn move_creatures(
    time: Res<Time>,
    food_query: Query<&Transform, (With<Food>, Without<Creature>)>,
    mut creature_query: Query<(
        &mut Transform,
        &mut Creature,
        &PreyTracker,
        &Personality,
    )>,
) {
    for (mut transform, mut creature, prey_tracker, personality) in &mut creature_query {
        let creature_pos = transform.translation.truncate();

        // Update hunger
        creature.hunger = (creature.hunger + time.delta_secs() * 0.1).min(1.0);

        let mut movement = Vec2::ZERO;

        // Use tracker system to find best food
        if creature.hunger > 0.3 {
            if let Some(best_food_entity) = prey_tracker.best_prey() {
                // Get position from entity
                if let Ok(food_transform) = food_query.get(best_food_entity) {
                    let food_pos = food_transform.translation.truncate();
                    let direction = (food_pos - creature_pos).normalize_or_zero();

                    // Speed influenced by hunger and personality
                    let base_speed = 30.0;
                    let hunger_urgency = creature.hunger * 25.0;
                    let assertiveness_boost = personality.resource_assertiveness * 15.0;
                    let speed = base_speed + hunger_urgency + assertiveness_boost;

                    movement = direction * speed;
                }
            }
        }

        // Random wandering if not seeking food
        if movement.length() < 5.0 {
            let t = time.elapsed_secs() + transform.translation.x * 0.01;
            movement = Vec2::new(t.sin() * 15.0, t.cos() * 15.0);
        }

        // Apply movement
        creature.velocity = creature.velocity.lerp(movement, 0.2);
        transform.translation.x += creature.velocity.x * time.delta_secs();
        transform.translation.y += creature.velocity.y * time.delta_secs();

        // Bounds
        let bounds = Vec2::new(300.0, 200.0);
        transform.translation.x = transform.translation.x.clamp(-bounds.x, bounds.x);
        transform.translation.y = transform.translation.y.clamp(-bounds.y, bounds.y);
    }
}

/// Handle food consumption and personality evolution
fn handle_food_consumption(
    mut creature_query: Query<(&Transform, &mut Creature, &mut Personality)>,
    mut food_query: Query<(&Transform, &mut Food, &mut Visibility)>,
) {
    for (creature_transform, mut creature, mut personality) in &mut creature_query {
        let creature_pos = creature_transform.translation.truncate();

        for (food_transform, mut food, mut visibility) in &mut food_query {
            if !food.active {
                continue;
            }

            let food_pos = food_transform.translation.truncate();
            if creature_pos.distance(food_pos) < 25.0 {
                // Consume food
                creature.hunger = 0.0;
                creature.food_consumed += 1;

                // Personality evolution from success
                personality.resource_assertiveness =
                    (personality.resource_assertiveness + 0.01).min(1.0);
                personality.competitive_strength =
                    (personality.competitive_strength + 0.005).min(1.0);

                food.active = false;
                food.respawn_timer.reset();
                *visibility = Visibility::Hidden;
                break;
            }
        }
    }
}

fn respawn_food(mut food_query: Query<(&mut Food, &mut Visibility)>, time: Res<Time>) {
    for (mut food, mut visibility) in &mut food_query {
        if !food.active {
            food.respawn_timer.tick(time.delta());
            if food.respawn_timer.is_finished() {
                food.active = true;
                *visibility = Visibility::Inherited;
            }
        }
    }
}

fn update_visuals(mut query: Query<(&Creature, &mut Sprite)>) {
    for (creature, mut sprite) in &mut query {
        let red = 0.2 + creature.hunger * 0.6;
        let green = 0.9 - creature.hunger * 0.3;
        sprite.color = Color::srgb(red, green, 0.2);
    }
}

fn update_labels(
    mut commands: Commands,
    creature_query: Query<(Entity, &Creature, &Transform, &Personality)>,
    mut label_query: Query<(Entity, &mut Text2d, &mut Transform, &CreatureLabel), Without<Creature>>,
) {
    for (label_entity, mut text, mut label_transform, label) in &mut label_query {
        if let Ok((_, creature, creature_transform, personality)) =
            creature_query.get(label.creature)
        {
            // Update position
            label_transform.translation.x = creature_transform.translation.x;
            label_transform.translation.y = creature_transform.translation.y + 25.0;

            // State based on hunger and personality
            let state = if creature.hunger > 0.8 {
                "Starving"
            } else if creature.hunger > 0.3 {
                if personality.resource_assertiveness > 0.6 {
                    "Competitive"
                } else {
                    "Seeking"
                }
            } else if personality.resource_assertiveness > 0.6 {
                "Evolved"
            } else {
                "Neutral"
            };

            **text = format!("{} ({})", state, creature.food_consumed);
        } else {
            commands.entity(label_entity).despawn();
        }
    }
}
