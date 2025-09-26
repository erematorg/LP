use super::oscillation::{WaveParameters, angular_frequency, wave_number};
use super::propagation::WavePosition;
use bevy::prelude::*;

#[inline]
fn normalize_or(vec: Vec2, fallback: Vec2) -> Vec2 {
    if vec.length_squared() > f32::EPSILON {
        vec.normalize()
    } else {
        fallback
    }
}

#[inline]
pub fn solve_standing_wave(
    params: &WaveParameters,
    position: Vec2,
    time: f32,
    interference_fn: Option<impl Fn(f32) -> f32>,
) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);

    let direction = normalize_or(params.direction, Vec2::X);
    let spatial_term = (k * direction.dot(position) + params.phase).sin();
    let temporal_term = (omega * time).cos();

    let damping_factor = (-params.damping * time).exp();

    let base_wave = params.amplitude * damping_factor * spatial_term * temporal_term;

    interference_fn
        .map(|f| base_wave * f(time))
        .unwrap_or(base_wave)
}

/// Marker component for standing waves
#[derive(Component, Reflect, Default)]
pub struct StandingWaveMarker;

/// System for updating standing waves specifically
pub fn update_standing_waves(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WaveParameters, &WavePosition), With<StandingWaveMarker>>,
) {
    let t = time.elapsed_secs();

    for (mut transform, params, position) in query.iter_mut() {
        let base_translation = Vec3::new(position.0.x, position.0.y, transform.translation.z);
        let displacement = solve_standing_wave(params, position.0, t, None::<fn(f32) -> f32>);
        let displacement_axis = normalize_or(params.displacement_axis, Vec2::Y);
        let displacement_vec = displacement_axis * displacement;
        transform.translation =
            base_translation + Vec3::new(displacement_vec.x, displacement_vec.y, 0.0);
    }
}

/// Event for standing wave modifications
#[derive(Event)]
pub struct StandingWaveModificationEvent {
    pub entity: Entity,
    pub new_parameters: WaveParameters,
}

/// System to handle wave parameter modifications
pub fn handle_wave_modifications(
    mut commands: Commands,
    mut wave_mod_events: EventReader<StandingWaveModificationEvent>,
) {
    for event in wave_mod_events.read() {
        commands.entity(event.entity).insert(event.new_parameters);
    }
}

/// Create wave parameters for standing wave calculation
pub fn create_standing_wave_parameters(
    amplitude: f32,
    wavelength: f32,
    frequency: f32,
    phase: f32,
    direction: Vec2,
    displacement_axis: Vec2,
    damping: f32,
    dispersion_factor: f32,
) -> WaveParameters {
    let defaults = WaveParameters::default();

    WaveParameters {
        amplitude,
        wavelength,
        speed: frequency * wavelength,
        phase,
        direction: normalize_or(direction, defaults.direction),
        displacement_axis: normalize_or(displacement_axis, defaults.displacement_axis),
        damping,
        dispersion_factor,
    }
}

/// Plugin for standing wave systems
pub struct StandingWavePlugin;

impl Plugin for StandingWavePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StandingWaveMarker>()
            .add_event::<StandingWaveModificationEvent>()
            .add_systems(Update, (update_standing_waves, handle_wave_modifications));
    }
}
