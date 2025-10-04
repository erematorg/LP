use bevy::prelude::*;

/// Trait for computing the squared norm of a vector efficiently
pub trait Norm {
    type Output;
    fn norm_squared(self) -> Self::Output;
}

/// Trait for computing the squared distance between vectors
pub trait Distance: Norm + std::ops::Sub<Output = Self> + Sized {
    fn distance_squared(self, other: Self) -> <Self as Norm>::Output {
        (self - other).norm_squared()
    }
}

// Implement for Vec3
impl Norm for Vec3 {
    type Output = f32;
    #[inline]
    fn norm_squared(self) -> f32 {
        self.length_squared()
    }
}

impl Distance for Vec3 {}

// Implement for Vec2
impl Norm for Vec2 {
    type Output = f32;
    #[inline]
    fn norm_squared(self) -> f32 {
        self.length_squared()
    }
}

impl Distance for Vec2 {}

/// Component for mass properties of an entity
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Mass {
    /// Mass in kilograms
    pub value: f32,
    /// Whether this object has infinite mass (immovable)
    pub is_infinite: bool,
}

impl Mass {
    pub fn new(value: f32) -> Self {
        debug_assert!(
            value > 0.0,
            "Mass cannot be negative or zero in real physics"
        );
        debug_assert!(
            value < 1e30,
            "Mass exceeds realistic bounds (solar mass ~2e30 kg)"
        );
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
            1.0 / self.value.max(f32::EPSILON)
        }
    }

    pub fn is_negligible(&self) -> bool {
        self.value < 0.001
    }

    pub fn reduced_mass(&self, other: &Mass) -> f32 {
        if self.is_infinite || other.is_infinite {
            return self.value.min(other.value);
        }

        let sum = self.value + other.value;
        if sum < f32::EPSILON {
            return 0.0;
        }

        (self.value * other.value) / sum
    }
}

/// Component representing a force applied to an entity
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
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

impl Default for AppliedForce {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
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

/// Component for velocity (both linear and angular)
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Velocity {
    /// Linear velocity in meters per second
    pub linvel: Vec3,
    /// Angular velocity in radians per second
    pub angvel: Vec3,
}

/// System to apply forces according to Newton's Second Law (F = ma)
pub fn apply_forces(time: Res<Time>, mut query: Query<(&Mass, &mut Velocity, &mut AppliedForce)>) {
    let dt = time.delta_secs();

    for (mass, mut velocity, mut force) in query.iter_mut() {
        if mass.is_infinite || mass.is_negligible() {
            continue;
        }

        if force.is_expired() {
            force.force = Vec3::ZERO;
            continue;
        }

        let acceleration = force.force * mass.inverse();

        // Cap extremely high accelerations to prevent instability
        let max_acceleration = 1000.0;
        let acceleration = if acceleration.norm_squared() > max_acceleration * max_acceleration {
            acceleration.normalize() * max_acceleration
        } else {
            acceleration
        };

        velocity.linvel += acceleration * dt;
        force.elapsed += dt;

        // Clear accumulated force so subsequent systems can rebuild it per-frame
        force.force = Vec3::ZERO;
    }
}

/// System to apply Verlet integration for position updates
pub fn integrate_positions(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.linvel * dt;

        if velocity.angvel.norm_squared() > 0.0 {
            transform.rotation *= Quat::from_scaled_axis(velocity.angvel * dt);
        }
    }
}

/// Calculate momentum of an object
pub fn calculate_momentum(mass: &Mass, velocity: &Velocity) -> Vec3 {
    mass.value * velocity.linvel
}

/// Calculate kinetic energy of an object
pub fn calculate_kinetic_energy(mass: &Mass, velocity: &Velocity) -> f32 {
    0.5 * mass.value * velocity.linvel.norm_squared()
}

/// Represents a pair of entities for force calculations (Newton's Third Law)
#[derive(Debug, Clone, Copy)]
pub struct ForcePair<'a> {
    pub first: (Entity, &'a Transform, &'a Mass),
    pub second: (Entity, &'a Transform, &'a Mass),
}

/// Trait for computing paired forces that satisfy Newton's Third Law
pub trait PairedForce {
    fn compute_pair_force(&self, pair: ForcePair) -> (Vec3, Vec3);
}

/// Component marker for entities that should be considered for paired force calculations
#[derive(Component)]
pub struct PairedForceInteraction;

/// Event for immediate impulse application that respects Newton's Third Law
#[derive(Message)]
pub struct ForceImpulse {
    pub entity1: Entity,
    pub impulse1: Vec3,
    pub entity2: Entity,
    pub impulse2: Vec3,
}

impl ForceImpulse {
    /// Create a balanced impulse pair (equal and opposite)
    pub fn new_balanced(entity1: Entity, entity2: Entity, impulse_on_first: Vec3) -> Self {
        Self {
            entity1,
            impulse1: impulse_on_first,
            entity2,
            impulse2: -impulse_on_first,
        }
    }
}

/// Plugin that adds Newton's Laws mechanics systems in the correct order
#[derive(Default)]
pub struct NewtonLawsPlugin;

impl Plugin for NewtonLawsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ForceImpulse>().add_systems(
            Update,
            (apply_forces, apply_impulses, integrate_positions).chain(),
        );
    }
}

/// System to compute paired forces and apply them to entities
pub fn compute_paired_forces<T: PairedForce + Resource>(
    paired_force: Res<T>,
    entities: Query<(Entity, &Transform, &Mass), With<PairedForceInteraction>>,
    mut forces: Query<&mut AppliedForce>,
) {
    for [(entity1, transform1, mass1), (entity2, transform2, mass2)] in entities.iter_combinations()
    {
        let pair = ForcePair {
            first: (entity1, transform1, mass1),
            second: (entity2, transform2, mass2),
        };

        let (force1, force2) = paired_force.compute_pair_force(pair);

        // Apply calculated forces
        if let Ok(mut force) = forces.get_mut(entity1) {
            force.force += force1;
        }

        if let Ok(mut force) = forces.get_mut(entity2) {
            force.force += force2;
        }
    }
}

/// System to apply impulses directly to velocities
pub fn apply_impulses(
    mut impulses: MessageReader<ForceImpulse>,
    mut velocities: Query<(&Mass, &mut Velocity)>,
) {
    for impulse in impulses.read() {
        // Apply to first entity
        if let Ok((mass, mut vel)) = velocities.get_mut(impulse.entity1) {
            if !mass.is_infinite {
                vel.linvel += impulse.impulse1 * mass.inverse();
            }
        }

        // Apply to second entity
        if let Ok((mass, mut vel)) = velocities.get_mut(impulse.entity2) {
            if !mass.is_infinite {
                vel.linvel += impulse.impulse2 * mass.inverse();
            }
        }
    }
}
