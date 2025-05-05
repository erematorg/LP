use bevy::prelude::*;

/// Wave parameters for configuring wave behavior
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct WaveParameters {
    /// Wave propagation speed (units/second)
    pub speed: f32,
    /// Initial amplitude (maximum displacement)
    pub amplitude: f32,
    /// Wavelength (distance between consecutive peaks)
    pub wavelength: f32,
    /// Initial phase offset (radians)
    pub phase: f32,
    /// Direction of propagation (normalized)
    pub direction: Vec2,
    /// What axis to displace along (normal to the wave plane)
    pub displacement_axis: Vec2,
    /// Damping coefficient (energy loss over time)
    pub damping: f32,
    /// Dispersion factor controlling wave frequency behavior
    pub dispersion_factor: f32,
}

/// Calculate wave number (spatial frequency)
#[inline]
pub fn wave_number(wavelength: f32) -> f32 {
    2.0 * std::f32::consts::PI / wavelength
}

/// Calculate angular frequency
#[inline]
pub fn angular_frequency(speed: f32, wave_number: f32) -> f32 {
    speed * wave_number
}

/// Calculate damping coefficient from half-life
#[inline]
pub fn damping_from_half_life(half_life: f32) -> f32 {
    if half_life <= 0.0 {
        return 0.0;
    }
    (2.0_f32.ln()) / half_life
}

/// Create a wave with modified parameters
pub fn create_wave(
    speed: f32, 
    amplitude: f32, 
    wavelength: f32,
    phase: f32,
    direction: Vec2,
    displacement_axis: Vec2,
    damping: f32,
    dispersion_factor: f32
) -> WaveParameters {
    WaveParameters {
        speed,
        amplitude,
        wavelength,
        phase,
        direction: direction.normalize(),
        displacement_axis: displacement_axis.normalize(),
        damping,
        dispersion_factor,
    }
}

/// Event for wave generation or modification
#[derive(Event, Debug)]
pub struct WaveGenerationEvent {
    /// Parameters of the generated wave
    pub parameters: WaveParameters,
    /// Optional source entity
    pub source: Option<Entity>,
}

/// System for wave parameter validation
pub fn validate_wave_parameters(
    mut wave_generation_events: EventReader<WaveGenerationEvent>,
) {
    for event in wave_generation_events.read() {
        // Validate wave parameters
        assert!(event.parameters.amplitude >= 0.0, "Wave amplitude must be non-negative");
        assert!(event.parameters.wavelength > 0.0, "Wavelength must be positive");
        assert!(event.parameters.speed >= 0.0, "Wave speed must be non-negative");
        
        // Optional: Log or handle invalid parameters
    }
}