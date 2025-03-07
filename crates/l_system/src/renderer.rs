use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rand::prelude::*;
use rand_core::{RngCore, SeedableRng};

/// Component for an L-System branch
#[derive(Component)]
struct Branch;

/// Structs to store random values as Bevy resources
#[derive(Resource)]
struct LSystemAngle(pub f32);

#[derive(Resource)]
struct LSystemScaling(pub f32);

#[derive(Resource)]
struct LSystemSegmentLength(pub f32);

/// Parameter for branch depth scaling
#[derive(Resource)]
struct LSystemDepthScaleFactor(pub f32);

/// Parameter for angle variation
#[derive(Resource)]
struct LSystemAngleVariation(pub f32);

/// Parameters for line thickness
#[derive(Resource)]
struct LSystemBaseThickness(pub f32);

#[derive(Resource)]
struct LSystemThicknessScaleFactor(pub f32);

/// Parameter for directional growth bias (phototropism)
#[derive(Resource)]
struct LSystemDirectionalBias(pub f32);

/// Random number generator as a resource
#[derive(Resource)]
struct LSystemRng(pub ChaCha8Rng);

/// Spawns the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Draws the L-System output dynamically
fn draw_lsystem(
    mut commands: Commands,
    symbols: Res<LSystemSymbols>,
    angle: Res<LSystemAngle>,
    scaling_factor: Res<LSystemScaling>,
    segment_length: Res<LSystemSegmentLength>,
    depth_scale_factor: Res<LSystemDepthScaleFactor>,
    angle_variation: Res<LSystemAngleVariation>,
    base_thickness: Res<LSystemBaseThickness>,
    thickness_scale_factor: Res<LSystemThicknessScaleFactor>,
    directional_bias: Res<LSystemDirectionalBias>,
    mut rng: ResMut<LSystemRng>,
) {
    let rotation_angle = angle.0;
    let line_length = segment_length.0 * scaling_factor.0;
    let scale_factor = depth_scale_factor.0;

    // Generate random variation for each branch using bevy_rand
    let varied_angle = if angle_variation.0 > 0.0 {
        // Generate random value between 0 and 1, then convert to -0.5 to 0.5 range
        let random_value = rng.0.next_u32() as f32 / u32::MAX as f32;
        let random_factor = random_value - 0.5; // Convert to -0.5 to 0.5
        angle_variation.0 * random_factor
    } else {
        0.0
    };
    
    let interpreter_output = crate::interpreter::interpret(
        &symbols.0, 
        rotation_angle, 
        line_length, 
        scale_factor,
        varied_angle,
        base_thickness.0,
        thickness_scale_factor.0,
        directional_bias.0
    ).expect("Failed to interpret L-System symbols");

    for (i, (start, end)) in interpreter_output.positions.iter().enumerate() {
        let thickness = interpreter_output.thicknesses[i];
        let shape = shapes::Line(*start, *end);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(Color::WHITE, thickness),
            Branch,
        ));
    }
}

/// Resource to store L-System symbols
#[derive(Resource)]
pub struct LSystemSymbols(pub String);

/// Bevy app to render the L-System
pub fn run_renderer(
    output: &str, 
    angle: f32, 
    scaling_factor: f32, 
    segment_length: f32, 
    depth_scale_factor: f32, 
    angle_variation: f32,
    base_thickness: f32,
    thickness_scale_factor: f32,
    directional_bias: f32
) {
    let lsystem_symbols = LSystemSymbols(output.to_string());
    
    // Create a random number generator with a random seed
    // Use the system time as a simple seed
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let rng = ChaCha8Rng::seed_from_u64(seed);

    App::new()
        .insert_resource(lsystem_symbols)
        .insert_resource(LSystemAngle(angle))
        .insert_resource(LSystemScaling(scaling_factor))
        .insert_resource(LSystemSegmentLength(segment_length))
        .insert_resource(LSystemDepthScaleFactor(depth_scale_factor))
        .insert_resource(LSystemAngleVariation(angle_variation))
        .insert_resource(LSystemBaseThickness(base_thickness))
        .insert_resource(LSystemThicknessScaleFactor(thickness_scale_factor))
        .insert_resource(LSystemDirectionalBias(directional_bias))
        .insert_resource(LSystemRng(rng))
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "L-System Renderer".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (setup_camera, draw_lsystem))
        .run();
}