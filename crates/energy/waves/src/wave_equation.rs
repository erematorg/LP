/// Core wave equation modeling for all wave types
/// 
/// This module implements the mathematical foundation for wave phenomena,
/// providing a dimension-agnostic approach to wave calculations.

use bevy::prelude::*;

/// The general form of the wave equation is:
/// 
/// ∂²u/∂t² = c² * ∇²u
/// 
/// Where:
/// - u is the wave amplitude as a function of position and time
/// - t is time
/// - ∇² is the Laplacian operator
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
    /// Direction of wave propagation (normalized)
    pub direction: Vec3,
    /// What axis to displace along (normal to the wave plane)
    pub displacement_axis: Vec3,
}

impl Default for WaveParameters {
    fn default() -> Self {
        Self {
            speed: 1.0,
            amplitude: 1.0,
            wavelength: 1.0,
            phase: 0.0,
            direction: Vec3::X, // Default propagation along x-axis
            displacement_axis: Vec3::Y, // Default displacement along y-axis
        }
    }
}

/// Component to store the position for wave calculations
#[derive(Component, Debug, Clone)]
pub struct WavePosition(pub Vec3);

impl WavePosition {
    pub fn new(position: Vec3) -> Self {
        Self(position)
    }
    
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self(Vec3::new(x, y, 0.0))
    }
    
    pub fn from_x(x: f32) -> Self {
        Self(Vec3::new(x, 0.0, 0.0))
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

/// Solve the wave equation for a position at time t
/// 
/// This implements a generalized traveling wave solution:
/// u(r,t) = A * sin(k⋅r - ωt + φ)
/// Where r is the position vector and k is the wave vector
#[inline]
pub fn solve_wave(params: &WaveParameters, position: Vec3, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    // Calculate the dot product of wave vector and position
    let k_vec = params.direction.normalize() * k;
    let dot_product = k_vec.dot(position);
    
    params.amplitude * (dot_product - omega * time + params.phase).sin()
}

/// System that updates entities with wave displacements
pub fn update_wave_displacements(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WaveParameters, &WavePosition)>,
) {
    let t = time.elapsed_secs();
    
    for (mut transform, params, position) in query.iter_mut() {
        let displacement = solve_wave(params, position.0, t);
        
        // Calculate displacement vector along the specified axis
        let displacement_vec = params.displacement_axis.normalize() * displacement;
        
        // Apply displacement to the transform
        transform.translation += displacement_vec;
    }
}

// Helper functions for common wave patterns

/// Create parameters for a linear wave propagating along a direction
pub fn create_linear_wave(
    amplitude: f32, 
    wavelength: f32, 
    speed: f32,
    phase: f32, 
    direction: Vec3, 
    displacement_axis: Vec3
) -> WaveParameters {
    WaveParameters {
        amplitude,
        wavelength,
        speed,
        phase,
        direction: direction.normalize(),
        displacement_axis: displacement_axis.normalize(),
    }
}

/// Solve for a radial/circular wave emanating from a center point
#[inline]
pub fn solve_radial_wave(
    params: &WaveParameters, 
    center: Vec3, 
    position: Vec3, 
    time: f32
) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    // Calculate distance from center (radial component)
    let distance = position.distance(center);
    
    // Amplitude decreases with distance for a radial wave (1/r falloff)
    let amplitude_factor = if distance > 0.001 {
        params.amplitude / distance.sqrt()
    } else {
        params.amplitude
    };
    
    amplitude_factor * (k * distance - omega * time + params.phase).sin()
}

/// Create parameters for a standing wave pattern
pub fn create_standing_wave(
    amplitude: f32,
    wavelength: f32,
    frequency: f32,
    phase: f32,
    direction: Vec3,
    displacement_axis: Vec3
) -> WaveParameters {
    // Standing waves don't propagate but oscillate in place
    // We'll handle the standing wave pattern in a separate solver
    WaveParameters {
        amplitude,
        wavelength,
        speed: frequency * wavelength, // For consistency, though unused directly
        phase,
        direction: direction.normalize(),
        displacement_axis: displacement_axis.normalize(),
    }
}

/// Solve for a standing wave pattern
#[inline]
pub fn solve_standing_wave(
    params: &WaveParameters,
    position: Vec3,
    time: f32
) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    // Standing wave is a product of spatial and temporal components
    let direction = params.direction.normalize();
    let spatial_term = (k * direction.dot(position) + params.phase).sin();
    let temporal_term = (omega * time).cos();
    
    params.amplitude * spatial_term * temporal_term
}