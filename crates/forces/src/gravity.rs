use bevy::prelude::*;
use crate::mechanics::{AppliedForce, Mass};

/// Modified gravitational constant for simulation scale
/// We use a much larger value than the real G (6.67430e-11) to make
/// the simulation visually interesting at screen scale
pub const GRAVITATIONAL_CONSTANT: f32 = 0.1;

/// Component for uniform gravitational field (like on Earth's surface)
#[derive(Resource, Debug, Clone, Copy)]
pub struct UniformGravity {
    /// Gravity direction and magnitude (e.g., Vec3(0.0, -9.81, 0.0) for Earth)
    pub acceleration: Vec3,
}

impl Default for UniformGravity {
    fn default() -> Self {
        // Default to Earth gravity
        Self {
            acceleration: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

/// Component for entities affected by gravity
#[derive(Component, Debug, Clone, Copy)]
pub struct GravityAffected;

/// Marker component for gravity field measurement points
#[derive(Component, Debug, Clone, Copy)]
pub struct GravityFieldMarker;

/// System to apply uniform gravity forces to entities
pub fn apply_uniform_gravity(
    gravity: Res<UniformGravity>,
    mut query: Query<(Entity, &Mass, &mut AppliedForce), With<GravityAffected>>,
    mut commands: Commands,
) {
    for (entity, mass, mut force) in query.iter_mut() {
        // Calculate force using F = mg
        let gravity_force = mass.value * gravity.acceleration;
        
        // Add to existing force
        force.force += gravity_force;
        
        // Track massive objects for gravitational analysis
        if mass.value > 1000.0 {
            commands.entity(entity).insert(MassiveBody);
        }
    }
}

/// Component for objects that generate gravitational attraction
#[derive(Component, Debug, Clone, Copy)]
pub struct GravitySource;

/// Marker for bodies with significant mass
#[derive(Component, Debug, Clone, Copy)]
pub struct MassiveBody;

/// System to calculate gravitational attraction between entities
pub fn calculate_gravitational_attraction(
    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
    mut affected_query: Query<(Entity, &Transform, &Mass, &mut AppliedForce), With<GravityAffected>>,
) {
    // For each gravity source
    for (source_entity, source_transform, source_mass) in query.iter() {
        // Calculate its effect on all affected entities
        for (affected_entity, affected_transform, affected_mass, mut force) in affected_query.iter_mut() {
            // Skip self-attraction
            if source_entity == affected_entity {
                continue;
            }
            
            // Calculate vector between entities
            let direction = source_transform.translation - affected_transform.translation;
            let distance_squared = direction.length_squared();
            
            // Add a small constant to avoid division by zero or extreme forces
            // This also helps stabilize very close orbits
            let safe_distance_squared = distance_squared.max(25.0);
            
            // Calculate force magnitude using Newton's Law of Universal Gravitation
            // F = G * (m1 * m2) / r²
            let force_magnitude = GRAVITATIONAL_CONSTANT * 
                (source_mass.value * affected_mass.value) / safe_distance_squared;
            
            // Calculate force vector
            let force_vector = direction.normalize() * force_magnitude;
            
            // Add to existing force
            force.force += force_vector;
        }
    }
}

/// Calculate orbital velocity for circular orbit
pub fn calculate_orbital_velocity(
    central_mass: f32,
    orbit_radius: f32,
) -> f32 {
    // v = sqrt(G * M / r)
    (GRAVITATIONAL_CONSTANT * central_mass / orbit_radius).sqrt()
}

/// Calculate initial velocity for an elliptical orbit with given eccentricity
pub fn calculate_elliptical_orbit_velocity(
    central_mass: f32,
    distance: f32,
    eccentricity: f32,
    is_periapsis: bool,
) -> f32 {
    let mu = GRAVITATIONAL_CONSTANT * central_mass;
    let semimajor_axis = distance / (1.0 - eccentricity * if is_periapsis { 1.0 } else { -1.0 });
    
    // Use the vis-viva equation: v² = GM(2/r - 1/a)
    (mu * (2.0 / distance - 1.0 / semimajor_axis)).sqrt()
}

/// Calculate escape velocity from a massive body
pub fn calculate_escape_velocity(
    central_mass: f32,
    distance: f32,
) -> f32 {
    // v_escape = sqrt(2 * G * M / r)
    (2.0 * GRAVITATIONAL_CONSTANT * central_mass / distance).sqrt()
}