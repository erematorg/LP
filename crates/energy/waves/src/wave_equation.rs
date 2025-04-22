/// Core wave equation modeling for different wave types
/// 
/// This module implements the mathematical foundation for all wave phenomena,
/// focusing on the wave equation and its solutions.

use bevy::prelude::*;

/// The general form of the one-dimensional wave equation is:
/// 
/// ∂²u/∂t² = c² * ∂²u/∂x²
/// 
/// Where:
/// - u is the wave amplitude as a function of position and time
/// - t is time
/// - x is position
/// - c is the wave propagation speed

/// Wave parameters for configuring wave behavior
#[derive(Component, Debug, Clone)]
pub struct WaveParameters {
    /// Wave propagation speed (units/second)
    pub speed: f32,
    /// Initial amplitude (maximum displacement)
    pub amplitude: f32,
    /// Wavelength (distance between consecutive peaks)
    pub wavelength: f32,
    /// Initial phase offset (radians)
    pub phase: f32,
}

impl Default for WaveParameters {
    fn default() -> Self {
        Self {
            speed: 1.0,
            amplitude: 1.0,
            wavelength: 1.0,
            phase: 0.0,
        }
    }
}

/// Calculate wave number (k) from wavelength
#[inline]
pub fn wave_number(wavelength: f32) -> f32 {
    2.0 * std::f32::consts::PI / wavelength
}

/// Calculate angular frequency (ω) from wave speed and wave number
#[inline]
pub fn angular_frequency(speed: f32, wave_number: f32) -> f32 {
    speed * wave_number
}

/// Solve the 1D wave equation for position x at time t
/// 
/// This implements a traveling wave solution to the wave equation
/// in the form: u(x,t) = A * sin(kx - ωt + φ)
#[inline]
pub fn solve_wave_1d(params: &WaveParameters, position: f32, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    params.amplitude * (k * position - omega * time + params.phase).sin()
}

/// System that updates entities with a WaveDisplacement component based on the wave equation
pub fn update_wave_displacements(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WaveParameters, &WavePosition)>,
) {
    let t = time.elapsed_secs();
    
    for (mut transform, params, position) in query.iter_mut() {
        let displacement = solve_wave_1d(params, position.0, t);
        
        // Apply displacement along the y-axis by default
        transform.translation.y = displacement;
    }
}

/// Component to mark an entity's base position along the wave
#[derive(Component, Debug, Clone)]
pub struct WavePosition(pub f32);