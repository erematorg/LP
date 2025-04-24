use bevy::prelude::*;
use energy::electromagnetism::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, update_field_lines)
        .run();
}

#[derive(Component)]
struct FieldLine;

#[derive(Component)]
struct Charge;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Create a grid of field line indicators
    for i in -5..=5 {
        for j in -5..=5 {
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0), // Red
                    custom_size: Some(Vec2::new(5.0, 20.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(i as f32 * 50.0, j as f32 * 50.0, 0.0)),
                FieldLine,
            ));
        }
    }

    // Point charge visualization
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.0), // Yellow
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        Transform::default(),
        Charge,
    ));
}

fn update_field_lines(
    time: Res<Time>,
    mut field_query: Query<(&mut Transform, &mut Sprite), With<FieldLine>>,
    mut charge_query: Query<&mut Transform, (With<Charge>, Without<FieldLine>)>,
) {
    // Update charge position
    if let Ok(mut charge_transform) = charge_query.get_single_mut() {
        charge_transform.translation = Vec3::new(
            100.0 * time.elapsed_secs().sin(),
            100.0 * time.elapsed_secs().cos(),
            0.0,
        );
    }

    // Get charge position
    let charge_pos = if let Ok(charge_transform) = charge_query.get_single() {
        charge_transform.translation
    } else {
        Vec3::ZERO
    };

    for (mut transform, mut sprite) in field_query.iter_mut() {
        // Calculate electric field at this point
        let field = ElectricField::from_point_charge(
            1.0, // Charge value
            charge_pos,
            transform.translation,
        );

        let field_vec = field.field * 5.0; // Scale for visualization
        let field_strength = field_vec.length();

        // Set arrow rotation to match field direction
        if field_strength > 0.01 {
            transform.rotation = Quat::from_rotation_z(field_vec.y.atan2(field_vec.x));
            sprite.custom_size = Some(Vec2::new(5.0, field_strength.min(50.0)));
        } else {
            sprite.custom_size = Some(Vec2::new(0.0, 0.0));
        }
    }
}
