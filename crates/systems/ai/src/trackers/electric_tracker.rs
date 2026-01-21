//! Electric field perception - creatures sense electromagnetic fields
//!
//! MPM-safe: reads ElectricField components (EM module owns them).
//! Examples: Electric fish (electroreception), sharks (ampullae of Lorenzini)

use bevy::prelude::*;

/// Creature's ability to sense electric fields
/// (Electroreception - rare but real in nature)
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ElectricSensor {
    /// How far creature can sense fields (meters)
    pub range: f32,

    /// Minimum field strength to detect (V/m or N/C)
    pub sensitivity: f32,
}

impl Default for ElectricSensor {
    fn default() -> Self {
        Self {
            range: 30.0,
            sensitivity: 0.1, // Can detect weak fields
        }
    }
}

impl ElectricSensor {
    /// Create sensor with custom range
    pub fn with_range(mut self, range: f32) -> Self {
        self.range = range;
        self
    }

    /// Set sensitivity threshold
    pub fn with_sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }
}

/// Creature's current electric field perception state
/// Updated each frame by electric_tracker_system
#[derive(Component, Debug, Default)]
pub struct ElectricTracker {
    /// Strongest field nearby: (entity, position, field_strength)
    pub strongest_field: Option<(Entity, Vec2, f32)>,

    /// Electric field vector at creature's position
    /// (superposition of all nearby fields)
    pub field_at_position: Vec2,

    /// Total field magnitude at creature position
    pub field_magnitude: f32,
}

impl ElectricTracker {
    /// Is creature sensing significant field?
    pub fn detects_field(&self) -> bool {
        self.field_magnitude > 0.01
    }

    /// Get direction of strongest field gradient
    pub fn field_direction(&self) -> Option<Vec2> {
        if self.field_at_position.length() > 0.01 {
            Some(self.field_at_position.normalize())
        } else {
            None
        }
    }

    /// Get direction toward strongest field source
    pub fn strongest_source_direction(&self, creature_pos: Vec2) -> Option<Vec2> {
        self.strongest_field
            .map(|(_, pos, _)| (pos - creature_pos).normalize_or_zero())
    }
}

/// System to update electric trackers based on nearby ElectricField components
/// Reads ElectricField from environment (EM module owns these)
pub fn update_electric_trackers(
    mut creatures: Query<(&Transform, &ElectricSensor, &mut ElectricTracker)>,
    electric_sources: Query<(Entity, &Transform, &energy::prelude::ElectricField)>,
) {
    for (creature_transform, sensor, mut tracker) in creatures.iter_mut() {
        let creature_pos = creature_transform.translation.truncate();

        // Reset tracker state
        tracker.strongest_field = None;
        tracker.field_at_position = Vec2::ZERO;
        tracker.field_magnitude = 0.0;

        let mut max_field_strength = 0.0;

        // Superpose all electric fields at creature position
        for (entity, transform, e_field) in electric_sources.iter() {
            let field_pos = transform.translation.truncate();
            let distance = creature_pos.distance(field_pos);

            if distance > sensor.range {
                continue; // Out of sensing range
            }

            // Field strength at this point
            let field_strength = e_field.strength();

            if field_strength < sensor.sensitivity {
                continue; // Field too weak to detect
            }

            // Superpose field vectors (EM superposition principle)
            tracker.field_at_position += e_field.field;

            // Track strongest source
            if field_strength > max_field_strength {
                max_field_strength = field_strength;
                tracker.strongest_field = Some((entity, field_pos, field_strength));
            }
        }

        // Calculate total field magnitude
        tracker.field_magnitude = tracker.field_at_position.length();
    }
}
