use bevy::prelude::*;

/// Stefan-Boltzmann constant (W/(m²·K⁴))
pub const STEFAN_BOLTZMANN: f32 = 5.67e-8;

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
        conductivity: f32,  // Thermal conductivity (W/(m·K))
        density: f32,       // Density (kg/m³)
        specific_heat: f32, // Specific heat capacity (J/(kg·K))
    ) -> Self {
        Self {
            value: conductivity / (density * specific_heat).max(f32::EPSILON),
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

/// Emissivity property for radiation calculations
#[derive(Component, Debug, Clone, Copy)]
pub struct Emissivity {
    /// Dimensionless value between 0.0 and 1.0
    /// 0.0 = perfect reflector, 1.0 = perfect emitter (black body)
    pub value: f32,
}

impl Emissivity {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }
}

/// Calculate heat transfer via radiation
pub fn heat_radiation(
    emitter_temp: f32,  // Temperature of emitting body (K)
    receiver_temp: f32, // Temperature of receiving body (K)
    area: f32,          // Surface area of emitting body (m²)
    emissivity: f32,    // Emissivity of emitting body (0.0-1.0)
    view_factor: f32,   // Geometric view factor (0.0-1.0)
) -> f32 {
    let t1_4 = emitter_temp.powi(4);
    let t2_4 = receiver_temp.powi(4);

    STEFAN_BOLTZMANN * emissivity * area * view_factor * (t1_4 - t2_4)
}