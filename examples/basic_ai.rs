use bevy::prelude::*;
use systems::ai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add our modular AI plugins
        .add_plugins((
            LPAIPlugin::default(),      // Core utility AI framework
            SocialPlugin,               // Social relationship system
            TrackerPlugin,              // Entity tracking system
            DrivesPlugin,               // Biological drives system
            PersonalityPlugin,          // Personality and behavioral traits
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_creatures_and_perception,
                handle_food_consumption,
                respawn_food,
                update_visuals,
                update_personality_labels,
            ),
        )
        .run();
}

#[derive(Component, Default)]
struct CreatureAction {
    action_attempts: u32,
    total_food_consumed: u32,
    last_action_result: ActionState,
}

#[derive(Component)]
struct Creature {
    perception: Perception,
    velocity: Vec2,
    lifespan: f32,
    hunger: f32,
    action: CreatureAction,
    using_concurrent_actions: bool,
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

    // Spawn creatures
    for i in 0..8 {
        let x = ((i % 4) as f32 - 1.5) * 80.0;
        let y = (i as f32 / 4.0).floor() * 100.0 - 50.0;

        let entity = commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.9, 0.2),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Visibility::default(),
            Creature {
                perception: Perception::new(200.0),
                velocity: Vec2::new(0.0, 0.0),
                lifespan: 30.0,
                hunger: 0.0,
                action: CreatureAction::default(),
                using_concurrent_actions: true,
            },
            // AI components - all start neutral, diverge through experience  
            Personality::default(),
        )).id();

        // Add dynamic state label text  
        commands.spawn((
            Text2d::new("Neutral"),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(x, y + 25.0, 1.0))
                .with_scale(Vec3::splat(0.5)), // Half the size
            CreatureLabel { creature: entity },
        ));

        let mut entity_commands = commands.entity(entity);

        // Add varied personality starting points and components
        if i % 2 == 1 {
            entity_commands.insert(Altruistic::default());
        }
        
        // Add context-aware utilities component to enable physics-based decision making
        entity_commands.insert(ContextAwareUtilities::default());
    }

    // Spawn food patches
    for i in 0..4 {
        for j in 0..4 {
            let x = (i as f32 - 1.5) * 120.0;
            let y = (j as f32 - 1.5) * 80.0;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.5, 0.0),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                Visibility::default(),
                Food {
                    respawn_timer: Timer::from_seconds(10.0, TimerMode::Once),
                    active: true,
                },
            ));
        }
    }
}

fn update_creatures_and_perception(
    mut commands: Commands,
    mut params: ParamSet<(
        Query<(Entity, &mut Transform, &mut Creature, Option<&Altruistic>, &Personality)>,
        Query<(Entity, &Transform, &Food)>,
    )>,
    time: Res<Time>,
) {
    // Get food info first
    let food_entities: Vec<(Entity, Vec3, bool)> = params
        .p1()
        .iter()
        .map(|(e, t, f)| (e, t.translation, f.active))
        .collect();

    let active_food: Vec<(Entity, Vec3)> = food_entities
        .iter()
        .filter(|(_, _, active)| *active)
        .map(|(e, pos, _)| (*e, *pos))
        .collect();

    // Get a list of creatures and their positions first for altruistic behavior
    let creatures: Vec<(Entity, Vec2, f32, bool)> = {
        let query = params.p0();
        query
            .iter()
            .map(|(entity, transform, creature, altruistic, _)| {
                (entity, transform.translation.truncate(), creature.hunger, altruistic.is_some())
            })
            .collect()
    };

    // Update perception and calculate movements
    let mut velocities = Vec::new();
    let mut to_despawn = Vec::new();

    {
        let mut query = params.p0();

        for (entity, transform, mut creature, altruistic, personality) in query.iter_mut() {
            // First update perception
            let position = transform.translation.truncate();

            let food_for_perception: Vec<(Entity, Vec2)> = active_food
                .iter()
                .map(|(e, pos)| (*e, pos.truncate()))
                .collect();

            creature
                .perception
                .update(position, &food_for_perception, time.elapsed_secs());

            // Update relationships using Relations system
            let current_tick = (time.elapsed_secs() * 10.0) as u64;

            for (food_entity, _) in &food_for_perception {
                // Add cooperation relationship using Relations
                commands.entity(entity).insert(SocialRelation {
                    target: *food_entity,
                    strength: RelationshipStrength::new(0.8 + creature.hunger * 0.2),
                    relationship_type: RelationshipType::Cooperation,
                    last_interaction_tick: current_tick,
                });
            }

            // Update lifespan and hunger with action tracking
            creature.lifespan -= time.delta_secs();
            creature.hunger = (creature.hunger + time.delta_secs() * 0.1).min(1.0);
            creature.action.action_attempts += 1;


            // Despawn if lifespan expires or hunger reaches maximum
            if creature.lifespan <= 0.0 || creature.hunger >= 1.0 {
                to_despawn.push(entity);
                continue;
            }

            // Movement logic
            let mut movement = Vec2::ZERO;

            if creature.hunger > 0.3 && !active_food.is_empty() {
                if creature.using_concurrent_actions {
                    // Future: Relations-based social approach will go here

                    // Strategy 2: Direct distance-based approach
                    let mut direct_target: Option<Vec2> = None;
                    let mut closest_dist = f32::MAX;

                    for (_, food_pos) in &active_food {
                        let dist = position.distance(food_pos.truncate());
                        if dist < closest_dist {
                            closest_dist = dist;
                            direct_target = Some(food_pos.truncate());
                        }
                    }

                    // Altruistic behavior using Altruistic component
                    if let Some(altruistic_trait) = altruistic {
                        if altruistic_trait.should_be_altruistic(creature.hunger) {
                            if let Some(target_pos) = direct_target {
                                for (other_entity, other_pos, other_hunger, _) in &creatures {
                                    if *other_entity != entity {
                                        let other_distance = other_pos.distance(target_pos);
                                        let self_distance = position.distance(target_pos);

                                        // If another creature is closer and hungrier, look for a different food
                                        if other_distance < self_distance
                                            && *other_hunger > creature.hunger + 0.2
                                        {
                                            // Find alternative food
                                            let mut alternative = None;
                                            let mut alt_dist = f32::MAX;

                                            for (_, food_pos) in &active_food {
                                                let fp = food_pos.truncate();
                                                if fp.distance(target_pos) > 50.0 {
                                                    // Different food
                                                    let dist = position.distance(fp);
                                                    if dist < alt_dist {
                                                        alt_dist = dist;
                                                        alternative = Some(fp);
                                                    }
                                                }
                                            }

                                            if let Some(alt_pos) = alternative {
                                                direct_target = Some(alt_pos);
                                                // Add positive relationship with the creature we helped using Relations
                                                commands.entity(entity).insert(SocialRelation {
                                                    target: *other_entity,
                                                    strength: RelationshipStrength::new(0.9),
                                                    relationship_type: RelationshipType::Cooperation,
                                                    last_interaction_tick: current_tick,
                                                });
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Use direct distance approach (Relations queries will be added in future systems)
                    if direct_target.is_some() {
                        let food_pos = direct_target.unwrap();
                        let direction = (food_pos - position).normalize_or_zero();
                        
                        // Movement speed influenced by evolved personality traits
                        let base_speed = 30.0;
                        let hunger_urgency = creature.hunger * 25.0;
                        let assertiveness_boost = personality.resource_assertiveness * 15.0;
                        let stress_modifier = 1.0 + (1.0 - personality.stress_tolerance) * 0.3;
                        let total_speed = (base_speed + hunger_urgency + assertiveness_boost) * stress_modifier;
                        
                        movement = direction * total_speed;
                    }
                } else {
                    // Simplified single-strategy logic - direct approach to nearest food
                    if let Some((_, food_pos)) = active_food.first() {
                        let direction = (food_pos.truncate() - position).normalize_or_zero();
                        movement = direction * (40.0 + 30.0 * creature.hunger);
                    }
                }
            }

            // Add random movement if not seeking food
            if movement.length() < 5.0 {
                let t = time.elapsed_secs() + transform.translation.x * 0.01;
                movement = Vec2::new(t.sin() * 15.0, t.cos() * 15.0);
            }

            // Apply velocity with smoothing
            creature.velocity = creature.velocity.lerp(movement, 0.2);
            velocities.push((entity, creature.velocity));
        }
    }

    // Apply movement in a separate query to avoid multiple mutable borrows
    {
        let mut query = params.p0();

        for (entity_id, velocity) in velocities {
            if let Ok((_, mut transform, _, _, _)) = query.get_mut(entity_id) {
                // Update position
                transform.translation.x += velocity.x * time.delta_secs();
                transform.translation.y += velocity.y * time.delta_secs();

                // Keep within bounds
                let bounds = Vec2::new(300.0, 200.0);
                if transform.translation.x.abs() > bounds.x {
                    transform.translation.x = transform.translation.x.signum() * bounds.x;
                }
                if transform.translation.y.abs() > bounds.y {
                    transform.translation.y = transform.translation.y.signum() * bounds.y;
                }
            }
        }
    }

    // Despawn entities with expired lifespan
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

fn handle_food_consumption(
    mut creature_query: Query<(&Transform, &mut Creature, &mut Personality)>,
    mut food_query: Query<(Entity, &Transform, &mut Food, &mut Visibility)>,
) {
    for (creature_transform, mut creature, mut personality) in creature_query.iter_mut() {
        let creature_pos = creature_transform.translation.truncate();

        for (_, food_transform, mut food, mut visibility) in food_query.iter_mut() {
            if !food.active {
                continue;
            }

            let food_pos = food_transform.translation.truncate();
            let distance = creature_pos.distance(food_pos);

            if distance < 25.0 {
                creature.lifespan = (creature.lifespan + 15.0).min(30.0);
                creature.hunger = 0.0;
                creature.action.total_food_consumed += 1;
                creature.action.last_action_result = ActionState::Success;

                // SUCCESS = PERSONALITY EVOLUTION! Less hardcoded, more emergent
                personality.resource_assertiveness = (personality.resource_assertiveness + 0.01).min(1.0);
                personality.competitive_strength = (personality.competitive_strength + 0.005).min(1.0);
                
                // SYSTEMIC AI: Traits affect each other creating emergent behavior patterns
                // Success builds confidence but also competitive drive
                personality.stress_tolerance = (personality.stress_tolerance + 0.003).min(1.0);
                
                // High assertiveness creates stress over time (aggressive behavior is taxing)
                if personality.resource_assertiveness > 0.7 {
                    personality.stress_tolerance = (personality.stress_tolerance - 0.001).max(0.0);
                }
                
                // Competitive strength influences assertiveness (winners become more bold)
                if personality.competitive_strength > 0.6 {
                    personality.resource_assertiveness = (personality.resource_assertiveness + 0.002).min(1.0);
                }

                food.active = false;
                food.respawn_timer.reset();
                *visibility = Visibility::Hidden;
                break;
            }
        }
    }
}

fn respawn_food(mut food_query: Query<(&mut Food, &mut Visibility)>, time: Res<Time>) {
    for (mut food, mut visibility) in food_query.iter_mut() {
        if !food.active {
            food.respawn_timer.tick(time.delta());

            if food.respawn_timer.finished() {
                food.active = true;
                *visibility = Visibility::Inherited;
            }
        }
    }
}

fn update_visuals(mut query: Query<(&Creature, &mut Sprite, Option<&Altruistic>)>) {
    for (creature, mut sprite, altruistic) in &mut query {
        // Base color based on hunger
        let mut red = 0.2 + creature.hunger * 0.6;
        let green = 0.9 - creature.hunger * 0.3;

        // Base blue for concurrent actions
        let mut blue = 0.2;
        if creature.using_concurrent_actions {
            blue += 0.2;
        }

        // Add purple tint for altruistic creatures
        if altruistic.is_some() {
            red += 0.1;
            blue += 0.1;
        }

        sprite.color = Color::srgb(red, green, blue);
    }
}

fn update_personality_labels(
    mut commands: Commands,
    creature_query: Query<(Entity, &Personality, &Creature, &Transform, Option<&Altruistic>)>,
    mut label_query: Query<(Entity, &mut Text2d, &mut Transform, &CreatureLabel), Without<Creature>>,
) {
    for (label_entity, mut text, mut label_transform, label) in &mut label_query {
        if let Ok((_, personality, creature, creature_transform, altruistic)) = creature_query.get(label.creature) {
            // Update label position to follow creature
            label_transform.translation.x = creature_transform.translation.x;
            label_transform.translation.y = creature_transform.translation.y + 25.0;
            
            // Core AI states showing emergent personality development
            let state = if creature.hunger > 0.8 {
                "Starving"
            } else if creature.action.last_action_result == ActionState::Success && altruistic.is_some() {
                "Altruistic"
            } else if creature.hunger > 0.3 {
                if personality.resource_assertiveness > 0.6 && personality.stress_tolerance < 0.4 {
                    "Aggressive"
                } else if personality.resource_assertiveness > 0.6 {
                    "Competitive" 
                } else {
                    "Seeking"
                }
            } else if personality.resource_assertiveness > 0.6 || personality.competitive_strength > 0.6 {
                "Evolved"
            } else if personality.stress_tolerance > 0.7 {
                "Calm"
            } else {
                "Neutral"
            };
            
            **text = state.to_string();
        } else {
            // Creature died, despawn the label
            commands.entity(label_entity).despawn();
        }
    }
}
