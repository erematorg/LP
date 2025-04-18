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
                // Action Lifecycle Hook: Log final action state
                println!(
                    "Creature despawning - Lifespan: {}, Hunger: {}, Action Attempts: {}, Total Food Consumed: {}, Last Action: {:?}", 
                    creature.lifespan, 
                    creature.hunger, 
                    creature.action.action_attempts,
                    creature.action.total_food_consumed,
                    creature.action.last_action_result
                );
                to_despawn.push(entity);
                continue;
            }
            
            // Movement logic
            let mut movement = Vec2::ZERO;
            
            if creature.hunger > 0.3 && !active_food.is_empty() {
                // Find best food based on relationships
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
                            
                            // Action Lifecycle Hook: Log action initiation
                            println!(
                                "Creature initiating food seek - Distance: {}, Relationship Score: {}", 
                                dist, 
                                score
                            );
                        }
                    }
                }
                
                if let Some(food_pos) = target_food {
                    let direction = (food_pos - position).normalize_or_zero();
                    movement = direction * (40.0 + 30.0 * creature.hunger);
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
                // Action Lifecycle Hook: Food Consumption
                println!(
                    "Food consumed - Distance: {}, Hunger before: {}, Lifespan gain: 15.0", 
                    distance, 
                    creature.hunger
                );
                
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
                // Action Lifecycle Hook: Food Respawn
                println!("Food respawned");
                
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
        sprite.color = Color::srgb(
            0.2 + creature.hunger * 0.6, 
            0.9 - creature.hunger * 0.3, 
            0.2
        );
    }
}