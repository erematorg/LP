use bevy::prelude::*;
use systems::ai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_creatures_and_perception,
            handle_food_consumption,
            respawn_food,
            update_visuals
        ))
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
    social_network: SocialNetwork,
    perception: Perception,
    velocity: Vec2,
    lifespan: f32,
    hunger: f32,
    action: CreatureAction,
    using_concurrent_actions: bool,
    // Simple field to track if this creature is altruistic
    altruistic: bool,
}

#[derive(Component)]
struct Food {
    respawn_timer: Timer,
    active: bool,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    // Spawn creatures
    for i in 0..8 {
        let x = ((i % 4) as f32 - 1.5) * 80.0;
        let y = (i as f32 / 4.0).floor() * 100.0 - 50.0;
        
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.9, 0.2),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Visibility::default(),
            Creature {
                social_network: SocialNetwork::default(),
                perception: Perception::new(200.0),
                velocity: Vec2::new(0.0, 0.0),
                lifespan: 30.0,
                hunger: 0.0,
                action: CreatureAction::default(),
                using_concurrent_actions: true,
                // Enable altruism for creatures with odd indices
                altruistic: i % 2 == 1,
            },
        ));
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
        Query<(Entity, &mut Transform, &mut Creature)>,
        Query<(Entity, &Transform, &Food)>,
    )>,
    time: Res<Time>,
) {
    // Get food info first
    let food_entities: Vec<(Entity, Vec3, bool)> = params.p1()
        .iter()
        .map(|(e, t, f)| (e, t.translation, f.active))
        .collect();
    
    let active_food: Vec<(Entity, Vec3)> = food_entities
        .iter()
        .filter(|(_, _, active)| *active)
        .map(|(e, pos, _)| (*e, *pos))
        .collect();
        
    // Get a list of creatures and their positions first for altruistic behavior
    let creatures: Vec<(Entity, Vec2, f32)> = {
        let query = params.p0();
        query.iter()
            .map(|(entity, transform, creature)| (
                entity, 
                transform.translation.truncate(),
                creature.hunger
            ))
            .collect()
    };
    
    // Update perception and calculate movements
    let mut velocities = Vec::new();
    let mut to_despawn = Vec::new();
    
    {
        let mut query = params.p0();
        
        for (entity, transform, mut creature) in query.iter_mut() {
            // First update perception
            let position = transform.translation.truncate();
            
            let food_for_perception: Vec<(Entity, Vec2)> = active_food
                .iter()
                .map(|(e, pos)| (*e, pos.truncate()))
                .collect();
            
            creature.perception.update(
                position,
                &food_for_perception,
                time.elapsed_secs()
            );
            
            // Update relationships
            let hunger = creature.hunger;
            let current_tick = (time.elapsed_secs() * 10.0) as u64;
            
            for (food_entity, _) in &food_for_perception {
                creature.social_network.add_or_update_relationship(
                    *food_entity,
                    RelationshipType::Cooperation,
                    0.8 + hunger * 0.2,
                    current_tick
                );
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
                    // Strategy 1: Social-based approach (existing logic)
                    let mut social_target = None;
                    let mut social_score = 0.0;
                    let food_relationships = creature.social_network.query_relationships(
                        Some(RelationshipType::Cooperation),
                        Some(0.5)
                    );
                    
                    for (food_entity, relation) in &food_relationships {
                        if let Some((_, food_pos)) = active_food.iter().find(|(e, _)| e == food_entity) {
                            let dist = position.distance(food_pos.truncate());
                            let score = relation.strength.value() * (1.0 - dist / 400.0);
                            
                            if score > social_score {
                                social_score = score;
                                social_target = Some(food_pos.truncate());
                            }
                        }
                    }
                    
                    // Strategy 2: Direct distance-based approach
                    let mut direct_target = None;
                    let mut closest_dist = f32::MAX;
                    
                    for (_, food_pos) in &active_food {
                        let dist = position.distance(food_pos.truncate());
                        if dist < closest_dist {
                            closest_dist = dist;
                            direct_target = Some(food_pos.truncate());
                        }
                    }
                    
                    // Simple altruistic behavior - if this creature is altruistic and not too hungry,
                    // check if other creatures are closer to the target food and hungrier
                    if creature.altruistic && creature.hunger < 0.7 {
                        if let Some(target_pos) = direct_target {
                            for (other_entity, other_pos, other_hunger) in &creatures {
                                if *other_entity != entity {
                                    let other_distance = other_pos.distance(target_pos);
                                    let self_distance = position.distance(target_pos);
                                    
                                    // If another creature is closer and hungrier, look for a different food
                                    if other_distance < self_distance && *other_hunger > creature.hunger + 0.2 {
                                        // Find alternative food
                                        let mut alternative = None;
                                        let mut alt_dist = f32::MAX;
                                        
                                        for (_, food_pos) in &active_food {
                                            let fp = food_pos.truncate();
                                            if fp.distance(target_pos) > 50.0 { // Different food
                                                let dist = position.distance(fp);
                                                if dist < alt_dist {
                                                    alt_dist = dist;
                                                    alternative = Some(fp);
                                                }
                                            }
                                        }
                                        
                                        if let Some(alt_pos) = alternative {
                                            direct_target = Some(alt_pos);
                                            closest_dist = alt_dist;
                                            // Add positive relationship with the creature we helped
                                            creature.social_network.add_or_update_relationship(
                                                *other_entity,
                                                RelationshipType::Cooperation,
                                                0.9, // Strong cooperative relationship
                                                current_tick
                                            );
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Race mode: Use the strategy with higher score
                    let social_value = social_score;
                    let direct_value = if closest_dist < f32::MAX { 1.0 - (closest_dist / 500.0) } else { 0.0 };
                    
                    if social_value > direct_value && social_target.is_some() {
                        let food_pos = social_target.unwrap();
                        let direction = (food_pos - position).normalize_or_zero();
                        movement = direction * (40.0 + 30.0 * creature.hunger);
                    } else if direct_target.is_some() {
                        let food_pos = direct_target.unwrap();
                        let direction = (food_pos - position).normalize_or_zero();
                        movement = direction * (40.0 + 30.0 * creature.hunger);
                    }
                } else {
                    // Original single-strategy logic
                    let food_relationships = creature.social_network.query_relationships(
                        Some(RelationshipType::Cooperation),
                        Some(0.5)
                    );
                    
                    let mut target_food = None;
                    let mut best_score = 0.0;
                    
                    for (food_entity, relation) in &food_relationships {
                        if let Some((_, food_pos)) = active_food.iter().find(|(e, _)| e == food_entity) {
                            let dist = position.distance(food_pos.truncate());
                            let score = relation.strength.value() * (1.0 - dist / 400.0);
                            
                            if score > best_score {
                                best_score = score;
                                target_food = Some(food_pos.truncate());
                            }
                        }
                    }
                    
                    if let Some(food_pos) = target_food {
                        let direction = (food_pos - position).normalize_or_zero();
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
            if let Ok((_, mut transform, _)) = query.get_mut(entity_id) {
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
    mut creature_query: Query<(&Transform, &mut Creature)>,
    mut food_query: Query<(Entity, &Transform, &mut Food, &mut Visibility)>,
) {
    for (creature_transform, mut creature) in creature_query.iter_mut() {
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
                
                food.active = false;
                food.respawn_timer.reset();
                *visibility = Visibility::Hidden;
                break;
            }
        }
    }
}

fn respawn_food(
    mut food_query: Query<(&mut Food, &mut Visibility)>,
    time: Res<Time>,
) {
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

fn update_visuals(
    mut query: Query<(&Creature, &mut Sprite)>,
) {
    for (creature, mut sprite) in &mut query {
        // Base color based on hunger
        let mut red = 0.2 + creature.hunger * 0.6;
        let green = 0.9 - creature.hunger * 0.3;
        
        // Base blue for concurrent actions
        let mut blue = 0.2;
        if creature.using_concurrent_actions {
            blue += 0.2;
        }
        
        // Add purple tint for altruistic creatures
        if creature.altruistic {
            red += 0.1;
            blue += 0.1;
        }
        
        sprite.color = Color::srgb(red, green, blue);
    }
}