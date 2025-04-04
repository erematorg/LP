use bevy::prelude::*;

/// Temperature component for thermal systems
#[derive(Component, Debug, Clone, Copy)]
pub struct Temperature {
    /// Temperature in Kelvin
    pub value: f32,
}

impl Temperature {
    pub fn new(kelvin: f32) -> Self {
        Self {
            value: kelvin.max(0.0),
        }
    }

    pub fn from_celsius(celsius: f32) -> Self {
        Self::new(celsius + 273.15)
    }
}

/// Thermal conductivity property
#[derive(Component, Debug, Clone, Copy)]
pub struct ThermalConductivity {
    /// W/(m·K)
    pub value: f32,
}

/// Thermal diffusivity property
#[derive(Component, Debug, Clone, Copy)]
pub struct ThermalDiffusivity {
    /// m²/s
    pub value: f32,
}

impl ThermalDiffusivity {
    /// Calculate thermal diffusivity
    pub fn calculate(
        conductivity: f32,   // Thermal conductivity (W/(m·K))
        density: f32,        // Density (kg/m³)
        specific_heat: f32,  // Specific heat capacity (J/(kg·K))
    ) -> Self {
        Self {
            value: conductivity / (density * specific_heat).max(f32::EPSILON)
        }
    }
}

/// Calculate heat transfer via conduction
pub fn heat_conduction(
    temp_diff: f32,    // Temperature difference (K)
    area: f32,         // Contact area (m²)
    distance: f32,     // Material thickness (m)
    conductivity: f32, // Thermal conductivity (W/(m·K))
) -> f32 {
    // q = k·A·ΔT/d (W)
    conductivity * area * temp_diff / distance.max(f32::EPSILON)
}