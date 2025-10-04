use bevy::prelude::*;

// Physical constants
pub const STEFAN_BOLTZMANN: f32 = 5.67e-8; // W/(m²·K⁴)

/// Temperature component for thermal systems
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Temperature {
    /// Temperature in Kelvin
    pub value: f32,
}

impl Temperature {
    pub fn new(kelvin: f32) -> Self {
        debug_assert!(
            kelvin >= 0.0,
            "Temperature below absolute zero violates thermodynamics"
        );
        debug_assert!(
            kelvin < 1e8,
            "Temperature exceeds realistic stellar core bounds (~1e8 K)"
        );
        Self {
            value: kelvin.max(0.0),
        }
    }

    pub fn from_celsius(celsius: f32) -> Self {
        Self::new(celsius + 273.15)
    }

    pub fn to_celsius(&self) -> f32 {
        self.value - 273.15
    }
}

/// Thermal conductivity property
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct ThermalConductivity {
    /// W/(m·K)
    pub value: f32,
}

/// Thermal diffusivity property
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
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

/// Emissivity property for radiation calculations
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
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

/// Event for thermal energy transfer between entities
#[derive(Message, Debug)]
pub struct ThermalTransferEvent {
    /// Source entity losing thermal energy
    pub source: Entity,
    /// Target entity receiving thermal energy
    pub target: Entity,
    /// Amount of heat transferred
    pub heat_flow: f32,
}

/// System for calculating heat conduction between entities
pub fn calculate_thermal_transfer(
    mut thermal_transfer_events: MessageWriter<ThermalTransferEvent>,
    query: Query<(Entity, &Temperature, &ThermalConductivity)>,
) {
    // Use query.iter_combinations() to efficiently compare all entities
    let mut combinations = query.iter_combinations();
    while let Some([(entity1, temp1, conduct1), (entity2, temp2, conduct2)]) =
        combinations.fetch_next()
    {
        let temp_diff: f32 = temp1.value - temp2.value;
        let area: f32 = 1.0; // Placeholder
        let distance: f32 = 1.0; // Placeholder

        let heat_flow: f32 =
            (conduct1.value + conduct2.value) / 2.0 * area * temp_diff / distance.max(f32::EPSILON);

        if heat_flow.abs() > f32::EPSILON {
            thermal_transfer_events.write(ThermalTransferEvent {
                source: if heat_flow > 0.0 { entity1 } else { entity2 },
                target: if heat_flow > 0.0 { entity2 } else { entity1 },
                heat_flow: heat_flow.abs(),
            });
        }
    }
}

/// Utility functions for thermal calculations
pub mod thermal_utils {
    use super::*;

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
}

/// Plugin for thermal system management
pub struct ThermalSystemPlugin;

impl Plugin for ThermalSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types for reflection
            .register_type::<Temperature>()
            .register_type::<ThermalConductivity>()
            .register_type::<ThermalDiffusivity>()
            .register_type::<Emissivity>()
            // Add thermal transfer event channel
            .add_message::<ThermalTransferEvent>()
            // Add system for thermal calculations
            .add_systems(Update, calculate_thermal_transfer);
    }
}
