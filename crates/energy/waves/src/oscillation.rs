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
    /// Direction of propagation (normalized)
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

#[inline]
pub fn wave_number(wavelength: f32) -> f32 {
    2.0 * std::f32::consts::PI / wavelength
}

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