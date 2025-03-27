use bevy::prelude::*;

/// Component for mass properties of an entity
#[derive(Component, Debug, Clone, Copy)]
pub struct Mass {
    /// Mass in kilograms
    pub value: f32,
    /// Whether this object has infinite mass (immovable)
    pub is_infinite: bool,
}

impl Mass {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.max(0.001), // Prevent zero or negative mass
            is_infinite: false,
        }
    }

    pub fn infinite() -> Self {
        Self {
            value: f32::MAX,
            is_infinite: true,
        }
    }
    
    pub fn inverse(&self) -> f32 {
        if self.is_infinite {
            0.0
        } else {
            1.0 / self.value
        }
    }
}

/// Component representing a force applied to an entity
#[derive(Component, Debug, Clone)]
pub struct AppliedForce {
    /// Force vector in Newtons
    pub force: Vec3,
    /// Application point relative to entity center
    pub application_point: Option<Vec3>,
    /// Duration the force is applied (None for continuous)
    pub duration: Option<f32>,
    /// Elapsed time since force began
    pub elapsed: f32,
}

impl AppliedForce {
    pub fn new(force: Vec3) -> Self {
        Self {
            force,
            application_point: None,
            duration: None,
            elapsed: 0.0,
        }
    }
    
    pub fn with_application_point(mut self, point: Vec3) -> Self {
        self.application_point = Some(point);
        self
    }
    
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.duration {
            self.elapsed >= duration
        } else {
            false
        }
    }
}

/// System to apply forces according to Newton's Second Law (F = ma)
pub fn apply_forces(
    time: Res<Time>,
    mut query: Query<(&Mass, &mut Velocity, &mut AppliedForce)>,
) {
    let dt = time.delta_seconds();
    
    for (mass, mut velocity, mut force) in query.iter_mut() {
        // Skip infinite mass objects
        if mass.is_infinite {
            continue;
        }
        
        // Calculate acceleration using F = ma
        let acceleration = force.force / mass.value;
        
        // Update velocity using acceleration
        velocity.linvel += acceleration * dt;
        
        // Update force duration
        force.elapsed += dt;
    }
}

/// System to apply Verlet integration for position updates
pub fn integrate_positions(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    let dt = time.delta_seconds();
    
    for (velocity, mut transform) in query.iter_mut() {
        // Update position using velocity
        transform.translation += velocity.linvel * dt;
        
        // Apply angular velocity
        if velocity.angvel.length_squared() > 0.0 {
            transform.rotation *= Quat::from_scaled_axis(velocity.angvel * dt);
        }
    }
}

/// Component for velocity (both linear and angular)
#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity {
    /// Linear velocity in meters per second
    pub linvel: Vec3,
    /// Angular velocity in radians per second
    pub angvel: Vec3,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        }
    }
}

/// Calculate momentum of an object
pub fn calculate_momentum(mass: &Mass, velocity: &Velocity) -> Vec3 {
    mass.value * velocity.linvel
}

/// Calculate kinetic energy of an object
pub fn calculate_kinetic_energy(mass: &Mass, velocity: &Velocity) -> f32 {
    0.5 * mass.value * velocity.linvel.length_squared()
}