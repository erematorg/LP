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

/// Spawns the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Draws the L-System output dynamically
fn draw_lsystem(
    mut commands: Commands,
    symbols: Res<LSystemSymbols>,
    angle: Res<LSystemAngle>, // FIXED: Now wrapped in a struct
    scaling_factor: Res<LSystemScaling>, // FIXED: Now wrapped in a struct
) {
    let rotation_angle = angle.0; // Extract the value
    let line_length = 20.0 * scaling_factor.0; // Scale line length dynamically

    let interpreter_output = crate::interpreter::interpret(&symbols.0, rotation_angle, line_length)
        .expect("Failed to interpret L-System symbols");

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
pub fn run_renderer(output: &str, angle: f32, scaling_factor: f32) {
    let lsystem_symbols = LSystemSymbols(output.to_string());

    App::new()
        .insert_resource(lsystem_symbols)
        .insert_resource(LSystemAngle(angle)) // FIXED: Wrapped in a struct
        .insert_resource(LSystemScaling(scaling_factor)) // FIXED: Wrapped in a struct
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
