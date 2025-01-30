use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

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

#[derive(Resource)]
struct LSystemThickness(pub f32); // NEW - Stores base thickness

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
    thickness: Res<LSystemThickness>, // NEW
) {
    let rotation_angle = angle.0;
    let line_length = segment_length.0 * scaling_factor.0;
    let mut depth = 0; // Track iteration depth

    let interpreter_output = crate::interpreter::interpret(&symbols.0, rotation_angle, line_length)
        .expect("Failed to interpret L-System symbols");

    for (start, end) in interpreter_output.positions {
        let taper_factor = 1.0 - (depth as f32 / 10.0); // Adjust taper based on depth
        let dynamic_thickness = thickness.0 * taper_factor.max(0.5); // Prevents too-thin branches

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Line(start, end)),
                ..default()
            },
            Stroke::new(Color::WHITE, dynamic_thickness), // Thickness now dynamic!
            Branch,
        ));

        depth += 1; // Increase depth as we go
    }
}

/// Resource to store L-System symbols
#[derive(Resource)]
pub struct LSystemSymbols(pub String);

/// Bevy app to render the L-System
pub fn run_renderer(output: &str, angle: f32, scaling_factor: f32, segment_length: f32, thickness: f32) {
    let lsystem_symbols = LSystemSymbols(output.to_string());

    App::new()
        .insert_resource(lsystem_symbols)
        .insert_resource(LSystemAngle(angle))
        .insert_resource(LSystemScaling(scaling_factor))
        .insert_resource(LSystemSegmentLength(segment_length))
        .insert_resource(LSystemThickness(thickness)) // NEW - Passing thickness
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
