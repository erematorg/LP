use super::interpreter::SymbolType;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

// Components
/// Component for an L-System branch
#[derive(Component)]
#[allow(dead_code)]
struct Branch {
    /// Type of this branch segment
    symbol_type: SymbolType, //TODO: Extends it later and truly get this to work as expected
}

// Resources for L-System parameters
#[derive(Resource)]
struct LSystemParams {
    angle: f32,
    scaling_factor: f32,
    segment_length: f32,
    depth_scale_factor: f32,
    angle_variation: f32,
    base_thickness: f32,
    thickness_scale_factor: f32,
    directional_bias: f32,
    angle_evolution_factor: f32,
}

/// Random number generator as a resource
#[derive(Resource)]
struct LSystemRng(pub u64);

/// Resource to store L-System symbols
#[derive(Resource)]
pub struct LSystemSymbols(pub String);

// Systems
/// Spawns the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Adjust thickness based on symbol type
fn adjust_thickness_for_symbol(thickness: f32, symbol_type: SymbolType) -> f32 {
    match symbol_type {
        SymbolType::Core => thickness * 1.5, // Thicker for core elements
        SymbolType::Bifurcation => thickness * 1.2, // Slightly thicker for branch points
        SymbolType::Segment => thickness,    // Standard thickness
        SymbolType::Legacy => thickness,     // Standard thickness
    }
}

/// Draws the L-System output dynamically
fn draw_lsystem(
    mut commands: Commands,
    symbols: Res<LSystemSymbols>,
    params: Res<LSystemParams>,
    mut rng: ResMut<LSystemRng>,
    time: Res<Time>,
) {
    // Calculate parameters
    let line_length = params.segment_length * params.scaling_factor;

    // Generate random variation for each branch using Bevy's built-in time as a seed
    let varied_angle = if params.angle_variation > 0.0 {
        // Update our simple RNG with time
        rng.0 = rng.0.wrapping_add(time.elapsed_secs_f64() as u64);

        // Generate a simple random value between 0.0 and 1.0
        let random_value = ((rng.0 >> 32) as f32) / u32::MAX as f32;
        let random_factor = random_value - 0.5;
        params.angle_variation * random_factor
    } else {
        0.0
    };

    // Interpret the L-System
    let interpreter_output = super::interpreter::interpret(
        &symbols.0,
        params.angle,
        line_length,
        params.depth_scale_factor,
        varied_angle,
        params.base_thickness,
        params.thickness_scale_factor,
        params.directional_bias,
        params.angle_evolution_factor,
    )
    .expect("Failed to interpret L-System symbols");

    // Draw the branches
    for i in 0..interpreter_output.positions.len() {
        let (start, end) = interpreter_output.positions[i];
        let base_thickness = interpreter_output.thicknesses[i];
        let symbol_type = interpreter_output.types[i];

        // Adjust thickness based on symbol type
        let adjusted_thickness = adjust_thickness_for_symbol(base_thickness, symbol_type);

        let shape = shapes::Line(start, end);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(Color::WHITE, adjusted_thickness),
            Branch { symbol_type },
        ));
    }
}

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
    directional_bias: f32,
    angle_evolution_factor: f32,
) {
    // Create a simple seed from the current time
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create the L-System parameters resource
    let params = LSystemParams {
        angle,
        scaling_factor,
        segment_length,
        depth_scale_factor,
        angle_variation,
        base_thickness,
        thickness_scale_factor,
        directional_bias,
        angle_evolution_factor,
    };

    // Build and run the Bevy app
    App::new()
        .insert_resource(LSystemSymbols(output.to_string()))
        .insert_resource(params)
        .insert_resource(LSystemRng(seed))
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
