use bevy::prelude::*;

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
    pub direction: Vec2,
    /// What axis to displace along (normal to the wave plane)
    pub displacement_axis: Vec2,
    /// Damping coefficient (energy loss over time)
    pub damping: f32,
}

impl Default for WaveParameters {
    fn default() -> Self {
        Self {
            speed: 1.0,
            amplitude: 1.0,
            wavelength: 1.0,
            phase: 0.0,
            direction: Vec2::X,
            displacement_axis: Vec2::Y,
            damping: 0.0,
        }
    }
}

/// Component to store position for wave calculations
#[derive(Component, Debug, Clone)]
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
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveType {
    Traveling,
    Radial,
    Standing,
}

#[inline]
pub fn wave_number(wavelength: f32) -> f32 {
    2.0 * std::f32::consts::PI / wavelength
}

#[inline]
pub fn angular_frequency(speed: f32, wave_number: f32) -> f32 {
    speed * wave_number
}

#[inline]
pub fn solve_wave(params: &WaveParameters, position: Vec2, time: f32) -> f32 {
    let k = wave_number(params.wavelength);
    let omega = angular_frequency(params.speed, k);
    
    let k_vec = params.direction.normalize() * k;
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
    let omega = angular_frequency(params.speed, k);
    
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

pub fn update_wave_displacements(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WaveParameters, &WavePosition, Option<&WaveType>)>,
    wave_centers: Query<(&Transform, &WaveCenterMarker)>,
) {
    let t = time.elapsed_secs();
    
    for (mut transform, params, position, wave_type) in query.iter_mut() {
        let displacement = match wave_type {
            Some(WaveType::Radial) => {
                // Find the nearest wave center
                let center = wave_centers.iter()
                    .map(|(t, _)| t.translation.truncate())
                    .min_by(|a, b| {
                        let dist_a = a.distance(position.0);
                        let dist_b = b.distance(position.0);
                        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(Vec2::ZERO);
                    
                solve_radial_wave(params, center, position.0, t)
            },
            Some(WaveType::Standing) => solve_standing_wave(params, position.0, t),
            _ => solve_wave(params, position.0, t),
        };
        
        let displacement_vec = params.displacement_axis.normalize() * displacement;
        transform.translation.x += displacement_vec.x;
        transform.translation.y += displacement_vec.y;
    }
}

// Marker component for wave centers (for radial waves)
#[derive(Component)]
pub struct WaveCenterMarker;

// Helper functions for common wave patterns
pub fn create_linear_wave(
    amplitude: f32, 
    wavelength: f32, 
    speed: f32,
    phase: f32, 
    direction: Vec2, 
    displacement_axis: Vec2,
    damping: f32
) -> WaveParameters {
    WaveParameters {
        amplitude,
        wavelength,
        speed,
        phase,
        direction: direction.normalize(),
        displacement_axis: displacement_axis.normalize(),
        damping,
    }
}

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

/// Calculate damping coefficient from half-life
#[inline]
pub fn damping_from_half_life(half_life: f32) -> f32 {
    if half_life <= 0.0 {
        return 0.0;
    }
    (2.0_f32.ln()) / half_life
}