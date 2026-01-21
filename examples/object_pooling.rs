use bevy::prelude::*;
use utils::{EntityPool, Pooled};

/// Example demonstrating EntityPool for recycling particle entities
///
/// Run: cargo run --example object_pooling

#[derive(Component)]
struct Particle {
    lifetime: f32,
}

#[derive(Resource)]
struct SpawnTimer(Timer);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(utils::UtilsPlugin)
        .insert_resource(EntityPool::new(100))
        .insert_resource(SpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (spawn_particles, update_particles, cleanup_particles).chain(),
        )
        .run();
}

fn setup(mut pool: ResMut<EntityPool>, mut commands: Commands) {
    // Prewarm pool to avoid allocations during gameplay
    pool.prewarm(&mut commands, 50);
    println!("Prewarmed pool with 50 entities");
}

fn spawn_particles(
    mut pool: ResMut<EntityPool>,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
) {
    // Spawn 10 particles every 0.1s (frame-rate independent)
    if timer.0.tick(time.delta()).just_finished() {
        for i in 0..10 {
            let entity = pool.acquire(&mut commands);
            commands.entity(entity).insert((
                Particle { lifetime: 2.0 },
                Transform::from_xyz(i as f32 * 10.0, 0.0, 0.0),
            ));
        }
    }
}

fn update_particles(mut query: Query<(Entity, &mut Particle), Without<Pooled>>, time: Res<Time>) {
    for (entity, mut particle) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();
        if particle.lifetime <= 0.0 {
            println!("Particle {:?} expired", entity);
        }
    }
}

fn cleanup_particles(
    mut pool: ResMut<EntityPool>,
    mut commands: Commands,
    query: Query<(Entity, &Particle), Without<Pooled>>,
) {
    for (entity, particle) in query.iter() {
        if particle.lifetime <= 0.0 {
            pool.release(&mut commands, entity);
            println!(
                "Returned {:?} to pool (available: {})",
                entity,
                pool.available_count()
            );
        }
    }
}
