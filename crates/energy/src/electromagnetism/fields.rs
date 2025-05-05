use bevy::prelude::*;

/// Constants for electromagnetic calculations
pub const COULOMB_CONSTANT: f32 = 8.99e9;
pub const MAGNETIC_CONSTANT_DIV_4PI: f32 = 1e-7;

/// Represents an electric field component
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct ElectricField {
    /// Magnitude and direction of the electric field
    pub field: Vec2,
    /// Position of the field
    pub position: Vec2,
}

impl ElectricField {
    /// Create a new electric field
    pub fn new(field: Vec2, position: Vec2) -> Self {
        Self { field, position }
    }

    /// Calculate field strength
    pub fn strength(&self) -> f32 {
        self.field.length()
    }

    /// Calculate the electric field from a point charge
    pub fn from_point_charge(charge: f32, charge_position: Vec2, field_position: Vec2) -> Self {
        let r = field_position - charge_position;
        let distance_squared = r.length_squared();

        if distance_squared < 1e-10 {
            return Self::new(Vec2::ZERO, field_position);
        }

        let direction = r.normalize();
        let magnitude = COULOMB_CONSTANT * charge / distance_squared;

        Self::new(direction * magnitude, field_position)
    }

    /// Combine two electric fields (superposition principle)
    pub fn superpose(&self, other: &ElectricField) -> Self {
        assert!(
            (self.position - other.position).length() < 1e-6,
            "Cannot superpose fields at different positions"
        );

        Self::new(self.field + other.field, self.position)
    }
}

/// Represents a magnetic field component
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct MagneticField {
    /// Magnitude and direction of the magnetic field
    pub field: Vec2,
    /// Position of the field
    pub position: Vec2,
}

impl MagneticField {
    /// Create a new magnetic field
    pub fn new(field: Vec2, position: Vec2) -> Self {
        Self { field, position }
    }

    /// Calculate field strength
    pub fn strength(&self) -> f32 {
        self.field.length()
    }

    /// Calculate the magnetic field from a current element
    pub fn from_current_element(
        current: f32,
        current_direction: Vec2,
        current_position: Vec2,
        field_position: Vec2,
    ) -> Self {
        let r = field_position - current_position;
        let distance = r.length();

        if distance < 1e-10 {
            return Self::new(Vec2::ZERO, field_position);
        }

        let r_unit = r / distance;

        // Biot-Savart law: dB = (μ₀/4π) * (I dl × r̂) / r²
        let field_magnitude = MAGNETIC_CONSTANT_DIV_4PI * current / (distance * distance);
        let field = Vec2::new(-r_unit.y, r_unit.x) * current_direction.length() * field_magnitude;

        Self::new(field, field_position)
    }

    /// Combine two magnetic fields (superposition principle)
    pub fn superpose(&self, other: &MagneticField) -> Self {
        assert!(
            (self.position - other.position).length() < 1e-6,
            "Cannot superpose fields at different positions"
        );

        Self::new(self.field + other.field, self.position)
    }
}

/// Event for field interactions
#[derive(Event, Debug)]
pub struct ElectromagneticFieldInteractionEvent {
    /// Source entity generating the field
    pub source: Entity,
    /// Target entity experiencing the field
    pub target: Entity,
    /// Interaction strength
    pub interaction_strength: f32,
}

/// System for calculating field interactions
pub fn calculate_field_interactions(
    mut field_interaction_events: EventWriter<ElectromagneticFieldInteractionEvent>,
    electric_fields: Query<(Entity, &ElectricField)>,
    magnetic_fields: Query<(Entity, &MagneticField)>,
) {
    // Electric field interactions
    for (source_entity, source_field) in electric_fields.iter() {
        for (target_entity, target_field) in electric_fields.iter() {
            if source_entity == target_entity { continue; }
            
            let interaction_strength = source_field.strength() * target_field.strength();
            
            if interaction_strength > f32::EPSILON {
                field_interaction_events.send(ElectromagneticFieldInteractionEvent {
                    source: source_entity,
                    target: target_entity,
                    interaction_strength,
                });
            }
        }
    }

    // Magnetic field interactions (similar logic)
    for (source_entity, source_field) in magnetic_fields.iter() {
        for (target_entity, target_field) in magnetic_fields.iter() {
            if source_entity == target_entity { continue; }
            
            let interaction_strength = source_field.strength() * target_field.strength();
            
            if interaction_strength > f32::EPSILON {
                field_interaction_events.send(ElectromagneticFieldInteractionEvent {
                    source: source_entity,
                    target: target_entity,
                    interaction_strength,
                });
            }
        }
    }
}

/// Plugin for electromagnetic field systems
pub struct ElectromagneticFieldPlugin;

impl Plugin for ElectromagneticFieldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types for reflection
            .register_type::<ElectricField>()
            .register_type::<MagneticField>()
            
            // Add electromagnetic field interaction event
            .add_event::<ElectromagneticFieldInteractionEvent>()
            
            // Add system for field interactions
            .add_systems(Update, calculate_field_interactions);
    }
}