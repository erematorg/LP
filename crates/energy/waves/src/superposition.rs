use bevy::prelude::*;
use crate::oscillation::{WaveParameters, wave_number, angular_frequency};
use crate::propagation::WavePosition;

#[inline]
pub fn solve_standing_wave(params: &WaveParameters, position: Vec2, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    let direction = params.direction.normalize();
    let spatial_term = (k * direction.dot(position) + params.phase).sin();
    let temporal_term = (omega * time).cos();
    
    let damping_factor = (-params.damping * time).exp();
    
    params.amplitude * damping_factor * spatial_term * temporal_term
}

/// System for updating standing waves specifically
pub fn update_standing_waves(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WaveParameters, &WavePosition), With<StandingWaveMarker>>,
) {
    let t = time.elapsed_secs();
    
    for (mut transform, params, position) in query.iter_mut() {
        let displacement = solve_standing_wave(params, position.0, t);
        let displacement_vec = params.displacement_axis.normalize() * displacement;
        transform.translation.x += displacement_vec.x;
        transform.translation.y += displacement_vec.y;
    }
}

/// Marker component for standing waves
#[derive(Component)]
pub struct StandingWaveMarker;

pub fn create_standing_wave(
    amplitude: f32,
    wavelength: f32,
    frequency: f32,
    phase: f32,
    direction: Vec2,
    displacement_axis: Vec2,
    damping: f32
) -> WaveParameters {
    WaveParameters {
        amplitude,
        wavelength,
        speed: frequency * wavelength,
        phase,
        direction: direction.normalize(),
        displacement_axis: displacement_axis.normalize(),
        damping,
    }
}