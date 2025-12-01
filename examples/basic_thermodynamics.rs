use bevy::prelude::*;
use energy::thermodynamics::thermal::ThermalSystemPlugin;
use energy::thermodynamics::prelude::*;

const GRID_SIZE: usize = 10;
const CELL_SIZE: f32 = 50.0;
const GRID_OFFSET_X: f32 = -225.0;
const GRID_OFFSET_Y: f32 = 225.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ThermalSystemPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .init_resource::<InputState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_visuals))
        .run();
}

#[derive(Resource, Default)]
struct InputState {
    hot_mode: bool,
}

#[derive(Component)]
struct GridCell {
    x: usize,
    y: usize,
}

#[derive(Component)]
struct InstructionText;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Instructions text
    commands.spawn((
        Text("Click: Set heat source\nSpace: Toggle hot/cold\nCurrent: Hot".into()),
        Transform::from_xyz(-390.0, 250.0, 0.0),
        InstructionText,
    ));

    // Create grid of cells
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let position = Vec3::new(
                GRID_OFFSET_X + (x as f32 * CELL_SIZE),
                GRID_OFFSET_Y - (y as f32 * CELL_SIZE),
                0.0,
            );

            commands.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(CELL_SIZE - 2.0, CELL_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(position),
                Temperature::from_celsius(20.0),
                ThermalConductivity { value: 100.0 },
                HeatCapacity::aluminum(0.01), // 10g of aluminum per cell
                GridCell { x, y },
            ));
        }
    }
}

fn handle_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut input_state: ResMut<InputState>,
    mut grid_cells: Query<(&GridCell, &mut Temperature, &Transform)>,
    mut text_query: Query<&mut Text, With<InstructionText>>,
) {
    // Toggle hot/cold mode with Space key
    if keyboard_input.just_pressed(KeyCode::Space) {
        input_state.hot_mode = !input_state.hot_mode;
        if let Ok(mut text) = text_query.single_mut() {
            *text = Text(format!(
                "Click: Set heat source\nSpace: Toggle hot/cold\nCurrent: {}",
                if input_state.hot_mode { "Hot" } else { "Cold" }
            ));
        }
    }

    // Handle mouse clicks
    if mouse_input.just_pressed(MouseButton::Left) {
        // Get window handle properly
        let Ok(window) = windows.single() else {
            return;
        };

        if let Some(cursor_position) = window.cursor_position() {
            // Get camera components properly
            let Ok((camera, camera_transform)) = camera_q.single() else {
                return;
            };

            if let Ok(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                for (_cell, mut temp, transform) in grid_cells.iter_mut() {
                    let cell_pos = transform.translation.truncate();
                    let half_size = CELL_SIZE / 2.0;

                    if world_position.x >= cell_pos.x - half_size
                        && world_position.x <= cell_pos.x + half_size
                        && world_position.y >= cell_pos.y - half_size
                        && world_position.y <= cell_pos.y + half_size
                    {
                        temp.value = if input_state.hot_mode {
                            Temperature::from_celsius(100.0).value
                        } else {
                            Temperature::from_celsius(0.0).value
                        };
                    }
                }
            }
        }
    }
}

fn update_visuals(mut grid_cells: Query<(&Temperature, &mut Sprite)>) {
    for (temp, mut sprite) in grid_cells.iter_mut() {
        let celsius = temp.value - 273.15;

        sprite.color = if celsius < 15.0 {
            // Cold (blue)
            let t = ((celsius - 0.0) / 15.0).clamp(0.0, 1.0);
            Color::srgb(0.0, 0.0, 0.4 + 0.6 * (1.0 - t))
        } else if celsius > 25.0 {
            // Hot (red)
            let t = ((celsius - 25.0) / 75.0).clamp(0.0, 1.0);
            Color::srgb(0.4 + 0.6 * t, 0.0, 0.0)
        } else {
            // Room temperature - black
            Color::BLACK
        };
    }
}
