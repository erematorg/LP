use super::oscillation::{WaveParameters, angular_frequency, wave_number};
use bevy::prelude::*;

#[inline]
fn normalize_or(vec: Vec2, fallback: Vec2) -> Vec2 {
    if vec.length_squared() > f32::EPSILON {
        vec.normalize()
    } else {
        fallback
    }
}

// Calculate modified angular frequency with dispersion
#[inline]
pub fn dispersive_angular_frequency(params: &WaveParameters, k: f32) -> f32 {
    if params.dispersion_factor == 0.0 {
        // No dispersion case - standard linear relationship
        angular_frequency(params.speed, k)
    } else {
        // Simple power law dispersion model: ω = ck^n where n = 1 + dispersion_factor
        // When dispersion_factor = 0, n = 1 (linear, no dispersion)
        // When dispersion_factor > 0, higher frequencies travel faster
        // When dispersion_factor < 0, higher frequencies travel slower
        let n = 1.0 + params.dispersion_factor;
        params.speed * k.powf(n)
    }
}

/// Component to store position for wave calculations
#[derive(Component, Debug, Clone, Reflect)]
pub struct WavePosition(pub Vec2);

impl WavePosition {
    pub fn new(position: Vec2) -> Self {
        Self(position)
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn from_x(x: f32) -> Self {
        Self(Vec2::new(x, 0.0))
    }
}

/// Wave type marker component
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum WaveType {
    Traveling,
    Radial,
    Standing,
}

// Marker component for wave centers (for radial waves)
#[derive(Component, Reflect)]
pub struct WaveCenterMarker;

#[inline]
pub fn solve_wave(params: &WaveParameters, position: Vec2, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = dispersive_angular_frequency(params, k);

    let direction = normalize_or(params.direction, Vec2::X);
    let k_vec = direction * k;
    let dot_product = k_vec.dot(position);

    // Calculate the phase argument for the wave function
    let phase = dot_product - omega * time + params.phase;

    // Apply damping over time
    let damping_factor = (-params.damping * time).exp();

    // Use sine as the wave function (can be generalized later)
    let wave_function = phase.sin();

    params.amplitude * damping_factor * wave_function
}

#[inline]
pub fn solve_radial_wave(params: &WaveParameters, center: Vec2, position: Vec2, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = dispersive_angular_frequency(params, k);

    // Calculate vector from center to position
    let displacement = position - center;
    let distance = displacement.length();

    // Calculate spatial decay (amplitude decreases with distance)
    let spatial_falloff = if distance > 0.001 {
        1.0 / distance.sqrt()
    } else {
        1.0
    };

    // Calculate the phase argument, potentially allowing for direction-dependent effects
    let phase = k * distance - omega * time + params.phase;

    // Apply damping over time
    let damping_factor = (-params.damping * time).exp();

    // Calculate the final wave displacement
    params.amplitude * spatial_falloff * damping_factor * phase.sin()
}

pub fn update_wave_displacements(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &WaveParameters,
        &WavePosition,
        Option<&WaveType>,
    )>,
    wave_centers: Query<(&Transform, &WaveCenterMarker)>,
) {
    let t = time.elapsed_secs();

    for (mut transform, params, position, wave_type) in query.iter_mut() {
        let base_translation = Vec3::new(position.0.x, position.0.y, transform.translation.z);

        let displacement = match wave_type {
            Some(WaveType::Radial) => {
                // Find the nearest wave center
                let center = wave_centers
                    .iter()
                    .map(|(t, _)| t.translation.truncate())
                    .min_by(|a, b| {
                        let dist_a = a.distance(position.0);
                        let dist_b = b.distance(position.0);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(Vec2::ZERO);

                solve_radial_wave(params, center, position.0, t)
            }
            // Standing waves should be handled by the superposition system
            Some(WaveType::Standing) => 0.0,
            _ => solve_wave(params, position.0, t),
        };

        let displacement_axis = normalize_or(params.displacement_axis, Vec2::Y);
        let displacement_vec = displacement_axis * displacement;
        transform.translation =
            base_translation + Vec3::new(displacement_vec.x, displacement_vec.y, 0.0);
    }
}

// Helper functions for common wave patterns
pub fn create_linear_wave(
    amplitude: f32,
    wavelength: f32,
    speed: f32,
    phase: f32,
    direction: Vec2,
    displacement_axis: Vec2,
    damping: f32,
    dispersion_factor: f32,
) -> WaveParameters {
    let current = WaveParameters::default();

    WaveParameters {
        amplitude,
        wavelength,
        speed,
        phase,
        direction: normalize_or(direction, current.direction),
        displacement_axis: normalize_or(displacement_axis, current.displacement_axis),
        damping,
        dispersion_factor,
    }
}
