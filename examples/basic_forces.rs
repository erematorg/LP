use bevy::prelude::*;
use forces::core::newton_laws::{Mass, Velocity, AppliedForce, apply_forces, integrate_positions};
use forces::core::gravity::{GravitySource, GravityAffected, calculate_gravitational_attraction};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "N-Body Gravitational Simulation".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            calculate_gravitational_attraction,
            apply_forces,
            integrate_positions,
            update_sprites.after(integrate_positions),
        ))
        .run();
}

// Components for visualization
#[derive(Component)]
struct CelestialBody {
    radius: f32,
    color: Color,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
    
    // Central star
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.7, 0.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Mass::new(100000.0),
        Velocity::default(),
        AppliedForce::new(Vec3::ZERO),
        GravitySource,
        GravityAffected,
        CelestialBody {
            radius: 25.0,
            color: Color::srgb(1.0, 0.7, 0.0),
        },
    ));
    
    // Planets
    spawn_planet(
        &mut commands,
        Vec3::new(0.0, 150.0, 0.0),
        Vec3::new(8.5, 0.0, 0.0),
        10.0,
        1000.0,
        Color::srgb(0.2, 0.6, 1.0),
    );
    
    spawn_planet(
        &mut commands,
        Vec3::new(0.0, -200.0, 0.0),
        Vec3::new(-7.5, 0.0, 0.0),
        15.0,
        2000.0,
        Color::srgb(0.8, 0.2, 0.2),
    );
    
    spawn_planet(
        &mut commands,
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 6.0, 0.0),
        8.0,
        800.0,
        Color::srgb(0.2, 0.8, 0.4),
    );
}

fn spawn_planet(
    commands: &mut Commands,
    position: Vec3,
    initial_velocity: Vec3,
    radius: f32,
    mass: f32,
    color: Color,
) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
            ..default()
        },
        Transform::from_translation(position),
        Mass::new(mass),
        Velocity { linvel: initial_velocity, angvel: Vec3::ZERO },
        AppliedForce::new(Vec3::ZERO),
        GravitySource,
        GravityAffected,
        CelestialBody {
            radius,
            color,
        },
    ));
}

fn update_sprites(mut query: Query<(&Transform, &CelestialBody, &mut Sprite)>) {
    for (_transform, body, mut sprite) in query.iter_mut() {
        // Update sprite size if needed (e.g., for dynamic size changes)
        sprite.custom_size = Some(Vec2::new(body.radius * 2.0, body.radius * 2.0));
        
        // Could add trail effects or other visual enhancements here
    }
}