use bevy::prelude::*;

/// Represents an electromagnetic wave component
#[derive(Debug, Component)]
pub struct ElectromagneticWave {
    /// Wave frequency in Hertz
    pub frequency: f32,
    /// Wave direction
    pub direction: Vec3,
    /// Electric field amplitude
    pub electric_amplitude: f32,
    /// Magnetic field amplitude
    pub magnetic_amplitude: f32,
    /// Phase of the wave
    pub phase: f32,
}

/// Material electromagnetic properties component
#[derive(Debug, Clone, Copy, Component)]
pub struct MaterialProperties {
    /// Electric permittivity
    pub permittivity: f32,
    /// Magnetic permeability
    pub permeability: f32,
    /// Electrical conductivity
    pub conductivity: f32,
}