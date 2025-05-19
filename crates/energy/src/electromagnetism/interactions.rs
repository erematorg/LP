use super::fields::{ElectricField, MagneticField};
use bevy::prelude::*;

// Speed of light (in m/s) constant physical value
//TODO: Making this cleaner later on to make units of measure dynamic rather than admiting 1 meter = 1 meter, same for seconds and much more
const C: f32 = 299_792_458.0;

/// Represents an electromagnetic wave component
#[derive(Debug, Component, Reflect)]
pub struct ElectromagneticWave {
    /// Wave frequency in Hertz
    pub frequency: f32,
    /// Wave direction
    pub direction: Vec2,
    /// Electric field amplitude
    pub electric_amplitude: f32,
    /// Magnetic field amplitude
    pub magnetic_amplitude: f32,
    /// Phase of the wave
    pub phase: f32,
    /// Wave number (2π/wavelength)
    pub wave_number: f32,
}

impl ElectromagneticWave {
    pub fn new(frequency: f32, direction: Vec2, electric_amplitude: f32, phase: f32) -> Self {
        // Calculate wavelength and wave number
        let wavelength = C / frequency;
        let wave_number = 2.0 * std::f32::consts::PI / wavelength;

        // Calculate magnetic amplitude (B = E/c for EM waves in vacuum)
        let magnetic_amplitude = electric_amplitude / C;

        Self {
            frequency,
            direction: direction.normalize(),
            electric_amplitude,
            magnetic_amplitude,
            phase,
            wave_number,
        }
    }

    /// Calculate the electric and magnetic fields at a position and time
    pub fn get_fields_at(&self, position: Vec2, time: f32) -> (ElectricField, MagneticField) {
        // Projection of position onto wave direction
        let pos_projection = self.direction.dot(position);

        // Phase at position and time
        let total_phase = self.wave_number * pos_projection
            - 2.0 * std::f32::consts::PI * self.frequency * time
            + self.phase;

        // Calculate phase factor
        let phase_factor = total_phase.sin();

        // Electric field is perpendicular to direction
        // We'll use a simple perpendicular vector for demonstration
        let e_direction = Vec2::new(-self.direction.y, self.direction.x);

        // Electric field
        let e_field = e_direction * (self.electric_amplitude * phase_factor);

        // Magnetic field is perpendicular to direction and "out of plane" in 2D
        // In 2D, we represent this using a vector perpendicular to the electric field
        let m_direction = Vec2::new(-e_direction.y, e_direction.x);
        let m_field = m_direction * (self.magnetic_amplitude * phase_factor);

        (
            ElectricField::new(e_field, position),
            MagneticField::new(m_field, position),
        )
    }
}

/// Material electromagnetic properties component
#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct MaterialProperties {
    /// Electric permittivity
    pub permittivity: f32,
    /// Magnetic permeability
    pub permeability: f32,
    /// Electrical conductivity
    pub conductivity: f32,
}

impl MaterialProperties {
    /// Create material properties for vacuum
    pub fn vacuum() -> Self {
        Self {
            permittivity: 8.85418782e-12, // ε₀ (F/m)
            permeability: 1.25663706e-6,  // μ₀ (H/m)
            conductivity: 0.0,            // σ (S/m)
        }
    }

    /// Create custom material properties
    pub fn new(permittivity: f32, permeability: f32, conductivity: f32) -> Self {
        Self {
            permittivity,
            permeability,
            conductivity,
        }
    }

    /// Calculate the refractive index of the material
    pub fn refractive_index(&self) -> f32 {
        // n = √(εᵣμᵣ) where εᵣ and μᵣ are relative permittivity and permeability
        let vacuum = Self::vacuum();
        let relative_permittivity = self.permittivity / vacuum.permittivity;
        let relative_permeability = self.permeability / vacuum.permeability;

        (relative_permittivity * relative_permeability).sqrt()
    }

    /// Calculate the speed of light in this material
    pub fn light_speed(&self) -> f32 {
        // v = c/n
        C / self.refractive_index()
    }
}