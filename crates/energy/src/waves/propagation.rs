use bevy::prelude::*;
use utils::{GridCell, SpatialGrid};

/// TODO: Wave grid-based solver (LP-1 feature, currently scaffolded)
/// Pending full wave equation implementation
#[derive(Resource, Deref, DerefMut)]
#[allow(dead_code)]
pub(crate) struct WaveGrid(pub(crate) SpatialGrid);

use super::normalize_or;
use super::oscillation::{WaveParameters, angular_frequency, wave_number};
use crate::conservation::{EnergyBalance, EnergyQuantity, EnergyTransaction, TransactionType};

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
#[reflect(Component)]
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
#[reflect(Component)]
pub enum WaveType {
    Traveling,
    Radial,
    Standing,
}

// Marker component for wave centers (for radial waves)
#[derive(Component, Reflect)]
#[reflect(Component)]
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

/// TODO: Grid cell attachment (wave grid solver, LP-1)
#[allow(dead_code)]
pub(crate) fn attach_grid_cells_to_wave_centers(
    mut commands: Commands,
    mut grid: ResMut<WaveGrid>,
    query: Query<(Entity, &Transform), (With<WaveCenterMarker>, Without<GridCell>)>,
) {
    for (entity, transform) in query.iter() {
        let position = transform.translation.truncate();
        let cell = grid.world_to_grid(position);
        grid.insert_in_cell(entity, cell);
        commands.entity(entity).insert(GridCell { cell });
    }
}

/// TODO: Wave grid update (wave equation solver, LP-1)
#[allow(dead_code)]
pub(crate) fn update_wave_grid(
    mut grid: ResMut<WaveGrid>,
    mut query: Query<
        (Entity, &Transform, &mut GridCell),
        (With<WaveCenterMarker>, Changed<Transform>),
    >,
) {
    for (entity, transform, mut cell) in query.iter_mut() {
        let position = transform.translation.truncate();
        let new_cell = grid.world_to_grid(position);
        if new_cell != cell.cell {
            grid.move_entity(entity, cell.cell, new_cell);
            cell.cell = new_cell;
        }
    }
}

/// System to track wave energy loss from damping and report to energy ledger
/// Runs before wave displacement updates
pub fn apply_wave_damping_with_energy(
    time: Res<Time>,
    mut wave_sources: Query<
        (Entity, &WaveParameters, Option<&mut EnergyQuantity>),
        With<WaveCenterMarker>,
    >,
    mut ledger_query: Query<&mut EnergyBalance>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 {
        return; // Skip first frame
    }

    let current_time = time.elapsed_secs();

    for (entity, params, energy_opt) in wave_sources.iter_mut() {
        if params.damping <= 0.0 {
            continue; // No damping, no energy loss
        }

        // Wave energy proportional to amplitude²
        // E = 0.5 * k * A² where k is a proportionality constant
        // For simplicity, use E = A² (k absorbed into amplitude units)
        let amplitude_now = params.amplitude * (-params.damping * current_time).exp();
        let amplitude_prev = params.amplitude * (-params.damping * (current_time - dt)).exp();

        let energy_now = 0.5 * amplitude_now * amplitude_now;
        let energy_prev = 0.5 * amplitude_prev * amplitude_prev;
        let energy_lost = energy_prev - energy_now;

        if energy_lost > f32::EPSILON {
            // Update wave's energy component if it has one
            if let Some(mut energy_quantity) = energy_opt {
                if energy_quantity.value >= energy_lost {
                    energy_quantity.value -= energy_lost;
                } else {
                    let _actual_loss = energy_quantity.value; // Can't lose more than we have
                    energy_quantity.value = 0.0;
                }
            }

            // Record transaction to this entity's ledger
            if let Ok(mut ledger) = ledger_query.get_mut(entity) {
                ledger.record_transaction(EnergyTransaction {
                    transaction_type: TransactionType::Output,
                    amount: energy_lost,
                    source: Some(entity),
                    destination: None, // TODO: Dissipated to heat (awaits MPM thermal coupling)
                    timestamp: current_time,
                    transfer_rate: energy_lost / dt,
                    duration: dt,
                });
            }
        }
    }
}

pub(crate) fn update_wave_displacements(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &WaveParameters,
            &WavePosition,
            Option<&WaveType>,
        ),
        Without<WaveCenterMarker>,
    >, // Disjoint from wave_centers query
    wave_centers: Query<&Transform, With<WaveCenterMarker>>,
) {
    let t = time.elapsed_secs();

    for (mut transform, params, position, wave_type) in query.iter_mut() {
        let base_translation = Vec3::new(position.0.x, position.0.y, transform.translation.z);

        let displacement = match wave_type {
            Some(WaveType::Radial) => {
                // Find nearest wave center (O(N) but typically few wave centers)
                let Some(center) = wave_centers
                    .iter()
                    .map(|t| t.translation.truncate())
                    .filter(|c| c.distance(position.0) < params.wavelength * 10.0)
                    .min_by(|a, b| {
                        let dist_a = a.distance(position.0);
                        let dist_b = b.distance(position.0);
                        dist_a
                            .partial_cmp(&dist_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                else {
                    continue; // Skip entities without nearby wave center
                };

                solve_radial_wave(params, center, position.0, t)
            }
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
