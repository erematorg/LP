use bevy::prelude::*;
use forces::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "N-Body Gravitational Simulation".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1)))
        .insert_resource(GravityParams::default().with_softening(10.0)) // Better softening value for stability
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                reset_forces,
                calculate_gravitational_attraction,
                apply_forces,
                integrate_positions,
                (update_sprites, keep_in_bounds),
            )
                .chain(),
        )
        .run();
}

// Components for visualization
#[derive(Component)]
#[allow(dead_code)]
struct CelestialBody {
    radius: f32,
    color: Color,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);

    // Central star with consistent mass
    let star_mass = 100000.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.7, 0.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Mass::new(star_mass),
        Velocity::default(),
        AppliedForce::new(Vec3::ZERO),
        GravitySource,
        GravityAffected,
        CelestialBody {
            radius: 25.0,
            color: Color::srgb(1.0, 0.7, 0.0),
        },
    ));

    // Planets with proper orbital velocities
    let distances = [150.0, 200.0, 300.0];
    let masses = [1000.0, 2000.0, 800.0];
    let radii = [10.0, 15.0, 8.0];
    let colors = [
        Color::srgb(0.2, 0.6, 1.0),
        Color::srgb(0.8, 0.2, 0.2),
        Color::srgb(0.2, 0.8, 0.4),
    ];

    for i in 0..3 {
        let distance = distances[i];
        let angle = (i as f32) * std::f32::consts::TAU / 3.0; // Distribute evenly

        // Calculate proper orbital velocity for a circular orbit
        let orbital_velocity = calculate_orbital_velocity(star_mass, distance);

        // Position planet around the star
        let pos = Vec3::new(distance * angle.cos(), distance * angle.sin(), 0.0);

        // Velocity vector perpendicular to position vector
        let vel = Vec3::new(
            -angle.sin() * orbital_velocity,
            angle.cos() * orbital_velocity,
            0.0,
        );

        spawn_planet(&mut commands, pos, vel, radii[i], masses[i], colors[i]);
    }
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
        Velocity {
            linvel: initial_velocity,
            angvel: Vec3::ZERO,
        },
        AppliedForce::new(Vec3::ZERO),
        GravitySource,
        GravityAffected,
        CelestialBody { radius, color },
    ));
}

// Reset forces to avoid accumulation
fn reset_forces(mut query: Query<&mut AppliedForce>) {
    for mut force in query.iter_mut() {
        force.force = Vec3::ZERO;
    }
}

fn update_sprites(mut query: Query<(&Transform, &CelestialBody, &mut Sprite)>) {
    for (_transform, body, mut sprite) in query.iter_mut() {
        // Update sprite size if needed (e.g., for dynamic size changes)
        sprite.custom_size = Some(Vec2::new(body.radius * 2.0, body.radius * 2.0));

        // Could add trail effects or other visual enhancements here
    }
}

// Keep celestial bodies within bounds of the window
fn keep_in_bounds(windows: Query<&Window>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    let Ok(window) = windows.single() else {
        return; // Exit early if we can't get the window
    };

    let width = window.width();
    let height = window.height();

    // Rest of the function remains the same...
    let bound_x = width / 2.0 - 50.0;
    let bound_y = height / 2.0 - 50.0;

    for (mut transform, mut velocity) in query.iter_mut() {
        // Bounce off edges
        if transform.translation.x.abs() > bound_x {
            velocity.linvel.x = -velocity.linvel.x * 0.8; // Dampen on bounce
            transform.translation.x = transform.translation.x.signum() * bound_x;
        }

        if transform.translation.y.abs() > bound_y {
            velocity.linvel.y = -velocity.linvel.y * 0.8; // Dampen on bounce
            transform.translation.y = transform.translation.y.signum() * bound_y;
        }
    }
}
