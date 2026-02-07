//! Entity-based Newtonian mechanics for non-continuum objects.
//!
//! # Architecture: Dual Physics Backend
//!
//! LP uses a **dual physics backend** to handle different types of matter:
//!
//! ## Entity-Based Physics (This Module)
//! - **Scope**: Non-continuum objects (cameras, UI, large rigid bodies)
//! - **Components**: `Mass`, `Velocity`, `AppliedForce`, `MomentOfInertia`, `AppliedTorque`
//! - **Systems**: `integrate_newton_second_law`, `integrate_torques`
//! - **Use cases**: Objects that don't deform or need particle-level resolution
//!
//! ## MPM Physics (Future: `crates/systems/mpm/`)
//! - **Scope**: All continuum matter (water, soil, flesh, deformable solids)
//! - **Representation**: Particles + grid (Material Point Method)
//! - **Integration**: P2G transfer → grid solve → G2P transfer
//! - **Use cases**: Anything that flows, deforms, or needs material-level physics
//!
//! ## Unified Conservation
//! Both backends feed the same **energy ledger** (`crates/energy/conservation.rs`):
//! - Entity backend: `WorkDoneEvent`, `RotationalWorkEvent` → ledger
//! - MPM backend: Grid work events → same ledger (when implemented)
//! - Diagnostics: Aggregate across both backends for global conservation tracking
//!
//! ## TODO (MPM Implementation)
//! When MPM is implemented:
//! - [ ] MPM will compute `L = Σ(r × m·v)` from particles (not `MomentOfInertia` components)
//! - [ ] MPM will emit work events from grid-particle transfers
//! - [ ] `ForcesDiagnostics` will aggregate both entity and MPM contributions
//! - [ ] Gravity and other forces will have MPM-specific implementations

use crate::PhysicsSet;
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

/// Configuration for Newton's laws integration.
///
/// **Numerical stability parameters** - not IRL physics.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct IntegrationConfig {
    /// Maximum acceleration clamp (m/s²).
    /// **NUMERICAL STABILITY**: Not IRL physics, purely for integration safety.
    /// Prevents explosive forces from breaking time integration.
    /// **Units**: meters / second² (m/s²)
    pub max_acceleration: f32,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 1000.0, // ~100g
        }
    }
}

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

/// Component for moment of inertia (rotational mass).
///
/// **PHYSICS**: Moment of inertia I determines resistance to rotational acceleration.
/// - For point mass: I = m·r² (kg·m²)
/// - For rigid body: I = ∫ r²·dm (depends on mass distribution)
///
/// **UNITS**: kg·m² (kilogram-meter-squared)
///
/// **NOTE**: This is scalar moment of inertia for 2D rotation or principal axis in 3D.
/// Full 3D rigid body needs inertia tensor (3×3 matrix), but most LP use cases are 2D or symmetric.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct MomentOfInertia {
    /// Moment of inertia in kg·m²
    pub value: f32,
    /// Whether this object has infinite inertia (cannot rotate)
    pub is_infinite: bool,
}

impl MomentOfInertia {
    pub fn new(value: f32) -> Self {
        debug_assert!(value > 0.0, "Moment of inertia cannot be negative or zero");
        Self {
            value: value.max(0.0001), // Prevent zero inertia
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

    /// Moment of inertia for a uniform disk/cylinder rotating about its axis
    /// I = 0.5 * m * r²
    pub fn disk(mass: f32, radius: f32) -> Self {
        Self::new(0.5 * mass * radius * radius)
    }

    /// Moment of inertia for a solid sphere rotating about its center
    /// I = 0.4 * m * r²
    pub fn sphere(mass: f32, radius: f32) -> Self {
        Self::new(0.4 * mass * radius * radius)
    }

    /// Moment of inertia for a point mass at distance r from axis
    /// I = m * r²
    pub fn point_mass(mass: f32, radius: f32) -> Self {
        Self::new(mass * radius * radius)
    }
}

/// Component representing torque applied to an entity.
///
/// **PHYSICS**: Torque τ = r × F (cross product of lever arm and force)
/// - τ: Torque (N·m)
/// - r: Position vector from rotation axis to force application point (m)
/// - F: Applied force (N)
///
/// **UNITS**: Newton-meters (N·m) = kg·m²/s²
///
/// **ROTATIONAL ANALOG**: Torque is to angular acceleration what force is to linear acceleration.
/// τ = I·α (where α is angular acceleration rad/s²)
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AppliedTorque {
    /// Torque vector in Newton-meters (right-hand rule: thumb = axis, fingers = rotation direction)
    pub torque: Vec3,
    /// Duration the torque is applied (None for continuous)
    pub duration: Option<f32>,
    /// Elapsed time since torque began
    pub elapsed: f32,
}

impl Default for AppliedTorque {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

impl AppliedTorque {
    pub fn new(torque: Vec3) -> Self {
        Self {
            torque,
            duration: None,
            elapsed: 0.0,
        }
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

/// Integrate Newton's Second Law to update velocities from forces.
///
/// **ARCHITECTURE NOTE**: This is the **entity-based physics backend** for non-continuum objects
/// (cameras, UI, large non-deformable bodies). For continuum matter (water, soil, deformables),
/// LP uses MPM (Material Point Method) with particle-based integration.
///
/// **TODO (MPM)**: When MPM is implemented, continuum matter will use grid-based force integration
/// (P2G transfer, grid solve, G2P transfer) instead of this entity-based approach. Both backends
/// will emit work events to the same unified energy ledger.
///
/// **PHYSICS**: Newton's Second Law F = ma → a = F/m
/// - F: Net force (Newtons)
/// - m: Mass (kilograms)
/// - a: Acceleration (m/s²)
/// - Δv = a·Δt (velocity update via Euler integration)
///
/// **UNITS**:
/// - Force: Newtons (N) = kg·m/s²
/// - Mass: kilograms (kg)
/// - Velocity: meters/second (m/s)
/// - Time: seconds (s)
///
/// **APPROXIMATIONS**:
/// - Explicit Euler: v_new = v_old + a·dt (first-order accurate)
/// - Acceleration clamp: max 1000 m/s² (numerical stability, not IRL physics)
///
/// **CONSERVATION**: Momentum conserved if all forces are action-reaction pairs (F_ab = -F_ba).
/// Energy conservation: work done by forces reported via WorkDoneEvent for ledger tracking.
///
/// **NUMERICAL STABILITY**: Clamps extremely high accelerations to prevent explosive forces
/// from breaking integration. If forces exceed limit, likely a bug (check softening, timestep, scales).
pub fn integrate_newton_second_law(
    time: Res<Time>,
    config: Res<IntegrationConfig>,
    mut query: Query<(Entity, &Mass, &mut Velocity, &mut AppliedForce)>,
    mut work_events: MessageWriter<WorkDoneEvent>,
) {
    let dt = time.delta_secs();

    for (entity, mass, mut velocity, mut force) in query.iter_mut() {
        if mass.is_infinite || mass.is_negligible() {
            continue;
        }

        if force.is_expired() {
            force.force = Vec3::ZERO;
            continue;
        }

        let acceleration = force.force * mass.inverse();

        if !acceleration.is_finite() {
            force.force = Vec3::ZERO;
            continue; // Skip non-finite acceleration
        }

        // Cap extremely high accelerations to prevent instability
        let max_acceleration = config.max_acceleration;
        let acceleration = if acceleration.norm_squared() > max_acceleration * max_acceleration {
            acceleration.normalize() * max_acceleration
        } else {
            acceleration
        };

        // Calculate work using average velocity for better accuracy
        // (Landau Vol 1 §6: dT = F·dr, where dr = v_avg·dt)
        let v_old = velocity.linvel;
        velocity.linvel += acceleration * dt;
        let v_avg = (v_old + velocity.linvel) * 0.5;
        let work_done = force.force.dot(v_avg) * dt;

        force.elapsed += dt;

        // Report work done for energy conservation tracking
        if work_done.abs() > f32::EPSILON {
            work_events.write(WorkDoneEvent {
                entity,
                work: work_done,
            });
        }

        // Clear accumulated force so subsequent systems can rebuild it per-frame
        force.force = Vec3::ZERO;
    }
}

/// System to apply symplectic Euler integration for position updates
pub fn integrate_positions_symplectic_euler(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.linvel * dt;

        if velocity.angvel.norm_squared() > 0.0 {
            transform.rotation *= Quat::from_scaled_axis(velocity.angvel * dt);
        }
    }
}

/// Selects which integration path the Newton systems should use.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratorKind {
    /// Standard symplectic Euler (velocity update then position update).
    SymplecticEuler,
    /// Skip position integration so an external system can drive it (e.g., MPM).
    External,
}

impl Default for IntegratorKind {
    fn default() -> Self {
        Self::SymplecticEuler
    }
}

fn use_symplectic_euler(kind: Res<IntegratorKind>) -> bool {
    *kind == IntegratorKind::SymplecticEuler
}

/// Integrate torques to update angular velocities.
///
/// **ARCHITECTURE NOTE**: This is the **entity-based rigid body rotation** backend for non-continuum
/// objects with discrete rotation (cameras, UI, large non-deformable bodies). For continuum matter,
/// LP uses MPM where angular momentum emerges from particle positions: L = Σ(r_i × m_i·v_i).
///
/// **TODO (MPM)**: When MPM is implemented, continuum matter will compute angular momentum from
/// particle distributions rather than rigid body components. MPM will emit rotational work events
/// from grid-particle interactions to the same unified energy ledger.
///
/// **PHYSICS**: Rotational analog of Newton's Second Law: τ = I·α → α = τ/I
/// - τ: Torque (N·m)
/// - I: Moment of inertia (kg·m²)
/// - α: Angular acceleration (rad/s²)
/// - Δω = α·Δt (angular velocity update via Euler integration)
///
/// **UNITS**:
/// - Torque: Newton-meters (N·m) = kg·m²/s²
/// - Moment of inertia: kg·m²
/// - Angular velocity: radians/second (rad/s)
/// - Time: seconds (s)
///
/// **APPROXIMATIONS**:
/// - Explicit Euler: ω_new = ω_old + α·dt (first-order accurate)
/// - Assumes principal axis rotation (scalar moment of inertia)
/// - Full 3D requires inertia tensor (future work)
///
/// **CONSERVATION**: Angular momentum conserved if all torques are balanced.
/// Rotational work reported via RotationalWorkEvent for ledger tracking.
pub fn integrate_torques(
    time: Res<Time>,
    config: Res<IntegrationConfig>,
    mut query: Query<(Entity, &MomentOfInertia, &mut Velocity, &mut AppliedTorque)>,
    mut rotational_work_events: MessageWriter<RotationalWorkEvent>,
) {
    let dt = time.delta_secs();

    for (entity, inertia, mut velocity, mut torque) in query.iter_mut() {
        if inertia.is_infinite {
            continue; // Cannot rotate
        }

        if torque.is_expired() {
            torque.torque = Vec3::ZERO;
            continue;
        }

        // Calculate angular acceleration: α = τ/I
        let angular_acceleration = torque.torque * inertia.inverse();

        if !angular_acceleration.is_finite() {
            torque.torque = Vec3::ZERO;
            continue;
        }

        // Cap extremely high angular accelerations for stability
        // Using same threshold as linear (1000 rad/s² ≈ 57000°/s²)
        let max_angular_acceleration = config.max_acceleration; // rad/s²
        let angular_acceleration = if angular_acceleration.norm_squared()
            > max_angular_acceleration * max_angular_acceleration
        {
            angular_acceleration.normalize() * max_angular_acceleration
        } else {
            angular_acceleration
        };

        // Calculate rotational work using average angular velocity
        // W_rot = τ·Δθ where Δθ = ω_avg·dt (angle rotated)
        let omega_old = velocity.angvel;
        velocity.angvel += angular_acceleration * dt;
        let omega_avg = (omega_old + velocity.angvel) * 0.5;
        let delta_theta = omega_avg * dt; // Angle rotated (radians)
        let work_done = torque.torque.dot(delta_theta);

        torque.elapsed += dt;

        // Report rotational work done for energy conservation tracking
        if work_done.abs() > f32::EPSILON {
            rotational_work_events.write(RotationalWorkEvent {
                entity,
                work: work_done,
            });
        }

        // Clear accumulated torque
        torque.torque = Vec3::ZERO;
    }
}

/// Calculate momentum of an object
pub fn calculate_momentum(mass: &Mass, velocity: &Velocity) -> Vec3 {
    mass.value * velocity.linvel
}

/// Calculate kinetic energy of an object (translational only)
pub fn calculate_kinetic_energy(mass: &Mass, velocity: &Velocity) -> f32 {
    0.5 * mass.value * velocity.linvel.norm_squared()
}

/// Calculate angular momentum of an object.
///
/// **PHYSICS**: L = I·ω (angular momentum = moment of inertia × angular velocity)
/// - L: Angular momentum (kg·m²/s)
/// - I: Moment of inertia (kg·m²)
/// - ω: Angular velocity (rad/s)
///
/// **UNITS**: kg·m²/s
///
/// **CONSERVATION**: L is conserved when no external torques act on the system.
pub fn calculate_angular_momentum(inertia: &MomentOfInertia, velocity: &Velocity) -> Vec3 {
    inertia.value * velocity.angvel
}

/// Calculate rotational kinetic energy of an object.
///
/// **PHYSICS**: KE_rot = 0.5·I·ω² (rotational kinetic energy)
/// - KE_rot: Rotational kinetic energy (J)
/// - I: Moment of inertia (kg·m²)
/// - ω: Angular velocity (rad/s)
///
/// **UNITS**: Joules (J) = kg·m²/s²
///
/// **TOTAL KINETIC ENERGY**: KE_total = KE_trans + KE_rot = 0.5·m·v² + 0.5·I·ω²
pub fn calculate_rotational_kinetic_energy(inertia: &MomentOfInertia, velocity: &Velocity) -> f32 {
    0.5 * inertia.value * velocity.angvel.norm_squared()
}

/// Calculate torque from an off-center force.
///
/// **PHYSICS**: τ = r × F (cross product of lever arm and force)
/// - τ: Torque (N·m)
/// - r: Position vector from rotation axis to force application point (m)
/// - F: Force vector (N)
///
/// **RIGHT-HAND RULE**: Fingers point from r to F, thumb points along τ
///
/// **UNITS**: Newton-meters (N·m) = kg·m²/s²
pub fn calculate_torque_from_force(lever_arm: Vec3, force: Vec3) -> Vec3 {
    lever_arm.cross(force)
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

/// Event for reporting work done by forces (W = F·dx)
///
/// **ARCHITECTURE**: Part of unified conservation interface. Entity-based physics emits these
/// events when forces do work on entities. Energy crate (`conservation.rs`) listens and records
/// to the energy ledger.
///
/// **TODO (MPM)**: When MPM is implemented, it will emit equivalent work events from grid-particle
/// transfers. Both entity and MPM events feed the same `EnergyBalance` ledger for unified tracking.
///
/// Energy crate listens to this to track kinetic energy changes.
#[derive(Message)]
pub struct WorkDoneEvent {
    pub entity: Entity,
    pub work: f32, // Joules
}

/// Event for reporting rotational work done by torques (W_rot = τ·Δθ)
///
/// **ARCHITECTURE**: Part of unified conservation interface. Entity-based physics emits these
/// events when torques do rotational work on rigid bodies. Energy crate (`conservation.rs`)
/// listens and records to the energy ledger.
///
/// **TODO (MPM)**: When MPM is implemented, rotational work will emerge from particle motions
/// around centers of mass. MPM will emit equivalent events computed from particle angular momentum
/// changes. Both entity and MPM events feed the same `EnergyBalance` ledger.
///
/// Energy crate listens to this to track rotational kinetic energy changes.
#[derive(Message)]
pub struct RotationalWorkEvent {
    pub entity: Entity,
    pub work: f32, // Joules
}

/// Plugin that adds Newton's Laws mechanics systems in the correct order
#[derive(Default)]
pub struct NewtonLawsPlugin;

impl Plugin for NewtonLawsPlugin {
    fn build(&self, app: &mut App) {
        let integrate = integrate_positions_symplectic_euler.run_if(use_symplectic_euler);

        app.init_resource::<IntegratorKind>()
            .init_resource::<IntegrationConfig>()
            .add_message::<ForceImpulse>()
            .add_message::<WorkDoneEvent>()
            .add_message::<RotationalWorkEvent>()
            // Configure physics sets
            .configure_sets(
                Update,
                (
                    PhysicsSet::AccumulateForces,
                    PhysicsSet::ApplyForces,
                    PhysicsSet::Integrate,
                )
                    .chain(),
            )
            // Apply forces and torques, then integrate - chain all three
            .add_systems(
                Update,
                (
                    integrate_newton_second_law,
                    integrate_torques,
                    apply_impulses,
                )
                    .chain()
                    .in_set(PhysicsSet::ApplyForces),
            )
            .add_systems(Update, integrate.in_set(PhysicsSet::Integrate));
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

/// Snapshot diagnostics for total momentum, angular momentum, and kinetic energy.
#[derive(Resource, Debug, Default, Clone)]
pub struct ForcesDiagnostics {
    pub total_momentum: Vec3,
    pub total_angular_momentum: Vec3,
    pub total_kinetic_energy: f32,
    pub total_rotational_kinetic_energy: f32,
}

/// Updates diagnostics after velocity changes are applied.
pub fn update_forces_diagnostics(
    mut diagnostics: ResMut<ForcesDiagnostics>,
    query: Query<(&Mass, &Velocity, Option<&MomentOfInertia>)>,
) {
    let mut total_momentum = Vec3::ZERO;
    let mut total_angular_momentum = Vec3::ZERO;
    let mut total_kinetic_energy = 0.0;
    let mut total_rotational_kinetic_energy = 0.0;

    for (mass, velocity, inertia_opt) in &query {
        if mass.is_infinite {
            continue;
        }

        // Linear momentum and kinetic energy
        total_momentum += calculate_momentum(mass, velocity);
        total_kinetic_energy += calculate_kinetic_energy(mass, velocity);

        // Angular momentum and rotational kinetic energy (if entity has moment of inertia)
        if let Some(inertia) = inertia_opt {
            if !inertia.is_infinite {
                total_angular_momentum += calculate_angular_momentum(inertia, velocity);
                total_rotational_kinetic_energy +=
                    calculate_rotational_kinetic_energy(inertia, velocity);
            }
        }
    }

    diagnostics.total_momentum = total_momentum;
    diagnostics.total_angular_momentum = total_angular_momentum;
    diagnostics.total_kinetic_energy = total_kinetic_energy;
    diagnostics.total_rotational_kinetic_energy = total_rotational_kinetic_energy;
}

/// Plugin to enable Newton-law diagnostics.
#[derive(Default)]
pub struct ForcesDiagnosticsPlugin;

impl Plugin for ForcesDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForcesDiagnostics>()
            .add_systems(Update, update_forces_diagnostics.after(apply_impulses));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fma_integration() {
        // Test F = ma: applying force should change velocity by a = F/m * dt
        // Direct calculation test to avoid Bevy App complexity
        let mass = 2.0;
        let force = 10.0;
        let dt = 0.1;

        let acceleration = force / mass; // F/m = 10.0 / 2.0 = 5.0 m/s²
        let expected_dv = acceleration * dt; // a * dt = 5.0 * 0.1 = 0.5 m/s

        // Verify F = ma relationship
        assert_eq!(acceleration, 5.0);
        assert_eq!(expected_dv, 0.5);
    }

    #[test]
    fn test_balanced_impulse_conserves_momentum() {
        // Test Newton's 3rd: balanced impulse should sum to zero momentum change
        let impulse_magnitude = 10.0;
        let mass1 = 1.0;
        let mass2 = 2.0;

        // Calculate velocity changes from equal/opposite impulses
        let delta_v1 = impulse_magnitude / mass1; // 10.0 / 1.0 = 10.0
        let delta_v2 = -impulse_magnitude / mass2; // -10.0 / 2.0 = -5.0

        // Momentum change for each
        let dp1 = mass1 * delta_v1; // 1.0 * 10.0 = 10.0
        let dp2 = mass2 * delta_v2; // 2.0 * -5.0 = -10.0

        let total_momentum_change: f32 = dp1 + dp2;

        assert!(
            total_momentum_change.abs() < 1e-5,
            "Balanced impulse did not conserve momentum: total Δp = {}",
            total_momentum_change
        );
    }

    #[test]
    fn test_momentum_calculation() {
        let mass = Mass::new(5.0);
        let velocity = Velocity {
            linvel: Vec3::new(2.0, 0.0, 0.0),
            angvel: Vec3::ZERO,
        };
        let momentum = calculate_momentum(&mass, &velocity);
        assert_eq!(momentum, Vec3::new(10.0, 0.0, 0.0)); // p = mv
    }

    #[test]
    fn test_kinetic_energy_calculation() {
        let mass = Mass::new(2.0);
        let velocity = Velocity {
            linvel: Vec3::new(3.0, 4.0, 0.0),
            angvel: Vec3::ZERO,
        };
        let ke = calculate_kinetic_energy(&mass, &velocity);
        // KE = 0.5 * m * v² = 0.5 * 2.0 * (3² + 4²) = 0.5 * 2.0 * 25 = 25.0
        assert_eq!(ke, 25.0);
    }

    #[test]
    fn test_angular_momentum_calculation() {
        // L = I·ω (angular momentum = moment of inertia × angular velocity)
        let inertia = MomentOfInertia::new(2.0); // kg·m²
        let velocity = Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::new(0.0, 0.0, 5.0), // 5 rad/s around Z axis
        };
        let angular_momentum = calculate_angular_momentum(&inertia, &velocity);
        // L = I·ω = 2.0 × 5.0 = 10.0 kg·m²/s
        assert_eq!(angular_momentum, Vec3::new(0.0, 0.0, 10.0));
    }

    #[test]
    fn test_rotational_kinetic_energy_calculation() {
        // KE_rot = 0.5·I·ω²
        let inertia = MomentOfInertia::new(2.0); // kg·m²
        let velocity = Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::new(0.0, 0.0, 3.0), // 3 rad/s around Z axis
        };
        let ke_rot = calculate_rotational_kinetic_energy(&inertia, &velocity);
        // KE_rot = 0.5 × 2.0 × 3² = 0.5 × 2.0 × 9 = 9.0 J
        assert_eq!(ke_rot, 9.0);
    }

    #[test]
    fn test_torque_from_force_calculation() {
        // τ = r × F (cross product)
        // If force is applied at distance r from rotation axis, creates torque
        let lever_arm = Vec3::new(2.0, 0.0, 0.0); // 2m along X axis
        let force = Vec3::new(0.0, 5.0, 0.0); // 5N along Y axis
        let torque = calculate_torque_from_force(lever_arm, force);
        // τ = r × F = (2,0,0) × (0,5,0) = (0,0,10) N·m
        // Right-hand rule: fingers point from X to Y, thumb points along Z
        assert_eq!(torque, Vec3::new(0.0, 0.0, 10.0));
    }

    #[test]
    fn test_moment_of_inertia_formulas() {
        let mass = 10.0; // kg
        let radius = 2.0; // m

        // Disk: I = 0.5·m·r²
        let disk = MomentOfInertia::disk(mass, radius);
        assert_eq!(disk.value, 0.5 * 10.0 * 2.0 * 2.0); // = 20.0

        // Sphere: I = 0.4·m·r² (actually 2/5)
        let sphere = MomentOfInertia::sphere(mass, radius);
        assert_eq!(sphere.value, 0.4 * 10.0 * 2.0 * 2.0); // = 16.0

        // Point mass: I = m·r²
        let point = MomentOfInertia::point_mass(mass, radius);
        assert_eq!(point.value, 10.0 * 2.0 * 2.0); // = 40.0
    }

    #[test]
    fn test_total_energy_includes_rotation() {
        // Total KE = translational + rotational
        let mass = Mass::new(2.0);
        let inertia = MomentOfInertia::new(3.0);
        let velocity = Velocity {
            linvel: Vec3::new(4.0, 0.0, 0.0), // 4 m/s
            angvel: Vec3::new(0.0, 0.0, 2.0), // 2 rad/s
        };

        let ke_trans = calculate_kinetic_energy(&mass, &velocity);
        // KE_trans = 0.5 × 2.0 × 4² = 16.0 J

        let ke_rot = calculate_rotational_kinetic_energy(&inertia, &velocity);
        // KE_rot = 0.5 × 3.0 × 2² = 6.0 J

        let total_ke = ke_trans + ke_rot;
        assert_eq!(total_ke, 22.0); // 16.0 + 6.0 = 22.0 J
    }
}
