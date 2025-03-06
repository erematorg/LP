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

/// Parameter for branch depth scaling
#[derive(Resource)]
struct LSystemDepthScaleFactor(pub f32);

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
) {
    let rotation_angle = angle.0;
    let line_length = segment_length.0 * scaling_factor.0;
    let scale_factor = depth_scale_factor.0;

    let interpreter_output = crate::interpreter::interpret(
        &symbols.0, 
        rotation_angle, 
        line_length, 
        scale_factor
    ).expect("Failed to interpret L-System symbols");

    for (start, end) in interpreter_output.positions {
        let shape = shapes::Line(start, end);
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Stroke::new(Color::WHITE, 2.0),
            Branch,
        ));
    }
}

/// Resource to store L-System symbols
#[derive(Resource)]
pub struct LSystemSymbols(pub String);

/// Bevy app to render the L-System
pub fn run_renderer(output: &str, angle: f32, scaling_factor: f32, segment_length: f32, depth_scale_factor: f32) {
    let lsystem_symbols = LSystemSymbols(output.to_string());

    App::new()
        .insert_resource(lsystem_symbols)
        .insert_resource(LSystemAngle(angle))
        .insert_resource(LSystemScaling(scaling_factor))
        .insert_resource(LSystemSegmentLength(segment_length))
        .insert_resource(LSystemDepthScaleFactor(depth_scale_factor))
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