use bevy::prelude::*;

/// Wave parameters for configuring wave behavior
#[derive(Component, Debug, Clone, Copy, Reflect)]
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
            dispersion_factor: 0.0,
        }
    }
}

impl WaveParameters {
    /// Builder-style method for creating custom wave parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Fluent interface for setting speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Fluent interface for setting amplitude
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    /// Fluent interface for setting wavelength
    pub fn with_wavelength(mut self, wavelength: f32) -> Self {
        self.wavelength = wavelength;
        self
    }

    /// Fluent interface for setting phase
    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    /// Fluent interface for setting direction
    pub fn with_direction(mut self, direction: Vec2) -> Self {
        self.direction = direction.normalize();
        self
    }

    /// Fluent interface for setting displacement axis
    pub fn with_displacement_axis(mut self, displacement_axis: Vec2) -> Self {
        self.displacement_axis = displacement_axis.normalize();
        self
    }

    /// Fluent interface for setting damping
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// Fluent interface for setting dispersion factor
    pub fn with_dispersion_factor(mut self, dispersion_factor: f32) -> Self {
        self.dispersion_factor = dispersion_factor;
        self
    }
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

/// Event for wave generation or modification
#[derive(Event, Debug)]
pub struct WaveGenerationEvent {
    /// Parameters of the generated wave
    pub parameters: WaveParameters,
    /// Optional source entity
    pub source: Option<Entity>,
}

/// System for wave parameter validation
pub fn validate_wave_parameters(mut wave_generation_events: EventReader<WaveGenerationEvent>) {
    for event in wave_generation_events.read() {
        // Validate wave parameters
        assert!(
            event.parameters.amplitude >= 0.0,
            "Wave amplitude must be non-negative"
        );
        assert!(
            event.parameters.wavelength > 0.0,
            "Wavelength must be positive"
        );
        assert!(
            event.parameters.speed >= 0.0,
            "Wave speed must be non-negative"
        );

        // Optional: Log or handle invalid parameters
    }
}
