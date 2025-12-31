//! Thermal perception - creatures sense temperature gradients
//!
//! MPM-safe: reads Temperature components (MPM will update them later).
//! No hardcoded material properties, purely perception layer.

use bevy::prelude::*;

/// Creature's ability to sense temperature
/// (Like how snakes have thermal pits, humans feel heat/cold)
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ThermalSensor {
    /// How far creature can sense temperature (meters)
    pub range: f32,

    /// Minimum temperature difference to detect (Kelvin)
    pub sensitivity: f32,

    /// Preferred comfortable temperature (Kelvin)
    /// (e.g., human ~295K, reptile ~303K when active)
    pub preferred_temp: f32,
}

impl Default for ThermalSensor {
    fn default() -> Self {
        Self {
            range: 50.0,
            sensitivity: 1.0,  // Can detect 1K difference
            preferred_temp: 293.0,  // Room temp default (~20°C)
        }
    }
}

impl ThermalSensor {
    /// Create sensor with custom comfort zone
    pub fn with_preferred_temp(mut self, temp_kelvin: f32) -> Self {
        self.preferred_temp = temp_kelvin;
        self
    }

    /// Set sensing range
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

/// Creature's current thermal perception state
/// Updated each frame by thermal_tracker_system
#[derive(Component, Debug, Default)]
pub struct ThermalTracker {
    /// Hottest nearby entity: (entity, position, temperature)
    pub hottest_nearby: Option<(Entity, Vec2, f32)>,

    /// Coldest nearby entity: (entity, position, temperature)
    pub coldest_nearby: Option<(Entity, Vec2, f32)>,

    /// Direction toward preferred temperature zone
    /// (points toward comfort - away from too hot/cold)
    pub gradient_direction: Vec2,

    /// Current discomfort level (0.0 = perfect, 1.0+ = extreme)
    /// Based on distance from preferred_temp
    pub discomfort: f32,
}

impl ThermalTracker {
    /// Is creature thermally comfortable?
    pub fn is_comfortable(&self) -> bool {
        self.discomfort < 0.3  // Low discomfort threshold
    }

    /// Should creature seek new thermal zone?
    pub fn needs_temperature_regulation(&self) -> bool {
        self.discomfort > 0.5
    }

    /// Get direction to move for thermal comfort
    pub fn comfort_seeking_direction(&self) -> Option<Vec2> {
        if self.gradient_direction.length() > 0.01 {
            Some(self.gradient_direction.normalize())
        } else {
            None
        }
    }
}

/// System to update thermal trackers based on nearby Temperature components
/// Reads Temperature from environment (MPM will update these later)
pub fn update_thermal_trackers(
    mut creatures: Query<(&Transform, &ThermalSensor, &mut ThermalTracker)>,
    heat_sources: Query<(Entity, &Transform, &energy::prelude::Temperature)>,
) {
    for (creature_transform, sensor, mut tracker) in creatures.iter_mut() {
        let creature_pos = creature_transform.translation.truncate();

        // Reset tracker state
        tracker.hottest_nearby = None;
        tracker.coldest_nearby = None;
        tracker.gradient_direction = Vec2::ZERO;

        let mut hottest_temp = f32::NEG_INFINITY;
        let mut coldest_temp = f32::INFINITY;
        let mut weighted_gradient = Vec2::ZERO;
        let mut current_creature_temp = sensor.preferred_temp;  // Assume creature at preferred temp if no data

        // Find nearby thermal entities
        for (entity, transform, temperature) in heat_sources.iter() {
            let heat_pos = transform.translation.truncate();
            let distance = creature_pos.distance(heat_pos);

            if distance > sensor.range {
                continue;  // Out of sensing range
            }

            let temp = temperature.value;
            let temp_diff = (temp - sensor.preferred_temp).abs();

            if temp_diff < sensor.sensitivity {
                continue;  // Temperature difference too small to detect
            }

            // Track hottest
            if temp > hottest_temp {
                hottest_temp = temp;
                tracker.hottest_nearby = Some((entity, heat_pos, temp));
            }

            // Track coldest
            if temp < coldest_temp {
                coldest_temp = temp;
                tracker.coldest_nearby = Some((entity, heat_pos, temp));
            }

            // Build gradient toward comfort
            // If too hot nearby → gradient points away
            // If too cold nearby → gradient points away
            let direction_to_source = (heat_pos - creature_pos).normalize_or_zero();
            let discomfort_at_source = (temp - sensor.preferred_temp).abs();

            if temp > sensor.preferred_temp {
                // Too hot - gradient points away from heat
                weighted_gradient -= direction_to_source * discomfort_at_source / (distance + 1.0);
            } else {
                // Too cold - gradient points away from cold
                weighted_gradient -= direction_to_source * discomfort_at_source / (distance + 1.0);
            }
        }

        // Calculate creature's current discomfort
        // (Could be influenced by nearby sources or assume creature is at preferred temp)
        let mut total_nearby_temp_influence = 0.0;
        let mut influence_count = 0.0;

        if let Some((_, pos, temp)) = tracker.hottest_nearby {
            let distance = creature_pos.distance(pos);
            if distance < sensor.range * 0.5 {  // Close influence
                total_nearby_temp_influence += temp * (1.0 - distance / (sensor.range * 0.5));
                influence_count += 1.0 - distance / (sensor.range * 0.5);
            }
        }

        if influence_count > 0.0 {
            current_creature_temp = total_nearby_temp_influence / influence_count;
        }

        tracker.discomfort = (current_creature_temp - sensor.preferred_temp).abs() / sensor.sensitivity;
        tracker.gradient_direction = weighted_gradient;
    }
}

/// System to link thermal perception to homeostasis needs
/// Updates need satisfaction based on thermal comfort
pub fn update_thermal_needs(
    mut creatures: Query<(&ThermalTracker, &mut crate::drives::needs::Need)>,
) {
    for (tracker, mut need) in creatures.iter_mut() {
        // Only update Homeostasis needs
        if need.need_type != crate::drives::needs::NeedType::Homeostasis {
            continue;
        }

        // Thermal comfort maps to homeostasis satisfaction
        // discomfort = 0.0 -> satisfaction = 1.0 (perfect comfort)
        // discomfort = 1.0+ -> satisfaction = 0.0 (extreme discomfort)
        let thermal_satisfaction = (1.0 - tracker.discomfort).clamp(0.0, 1.0);

        // Blend with existing satisfaction (allows other homeostasis factors)
        need.satisfaction = need.satisfaction * 0.5 + thermal_satisfaction * 0.5;
    }
}
