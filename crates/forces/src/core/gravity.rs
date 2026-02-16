use super::newton_laws::{AppliedForce, Mass};
use crate::PhysicsSet;
use bevy::prelude::*;

// Simulation constants
// NOTE: Sim-tuned G (not SI 6.67e-11). Pixel→meter mapping TBD; adjust after scale is fixed.
pub const DEFAULT_GRAVITATIONAL_CONSTANT: f32 = 0.1;
/// Practical LP-0 guideline for exact mutual O(N^2) gravity in realtime.
pub const MUTUAL_REALTIME_BODY_LIMIT: usize = 100;

/// Resource for gravity simulation parameters
#[derive(Resource, Clone, Debug)]
pub struct GravityParams {
    /// Softening parameter to prevent singularities
    pub softening: f32,
    /// Gravitational constant controlling attraction strength
    pub gravitational_constant: f32,
    /// Maximum depth for Barnes-Hut octree spatial partitioning
    pub barnes_hut_max_depth: usize,
    /// Maximum bodies per node before subdivision in Barnes-Hut algorithm
    pub barnes_hut_max_bodies_per_node: usize,
}

impl Default for GravityParams {
    fn default() -> Self {
        Self {
            softening: 5.0,
            gravitational_constant: DEFAULT_GRAVITATIONAL_CONSTANT,
            barnes_hut_max_depth: 8,
            barnes_hut_max_bodies_per_node: 8,
        }
    }
}

/// Selects how gravitational forces are applied.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GravityForceMode {
    /// Apply forces only to affected bodies (one-way).
    OneWay,
    /// Apply equal and opposite forces between participating bodies.
    Mutual,
}

impl Default for GravityForceMode {
    fn default() -> Self {
        Self::OneWay
    }
}

impl GravityParams {
    pub fn with_softening(mut self, softening: f32) -> Self {
        self.softening = softening;
        self
    }

    pub fn with_gravitational_constant(mut self, gravitational_constant: f32) -> Self {
        self.gravitational_constant = gravitational_constant;
        self
    }

    pub fn with_barnes_hut_params(mut self, max_depth: usize, max_bodies_per_node: usize) -> Self {
        self.barnes_hut_max_depth = max_depth.max(1);
        self.barnes_hut_max_bodies_per_node = max_bodies_per_node.max(1);
        self
    }
}

/// Component for uniform gravitational field (like on Earth's surface)
#[derive(Resource, Debug, Clone, Copy)]
pub struct UniformGravity {
    pub acceleration: Vec3,
}

impl Default for UniformGravity {
    fn default() -> Self {
        Self {
            acceleration: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

/// Component for entities affected by gravity
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct GravityAffected;

/// Marker component for gravity field measurement points
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct GravityFieldMarker;

/// Component for objects that generate gravitational attraction
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct GravitySource;

/// Marker for bodies with significant mass
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct MassiveBody;

#[derive(Clone, Copy, Debug)]
struct GravityBody {
    entity: Entity,
    position: Vec3,
    mass: f32,
}

#[derive(Default)]
struct StagedGravitySources {
    entities: Vec<Entity>,
    positions: Vec<Vec3>,
    masses: Vec<f32>,
}

impl StagedGravitySources {
    /// Clear buffers and re-stage from the source query (reuses heap allocations).
    fn stage(&mut self, query: &Query<(Entity, &Transform, &Mass), With<GravitySource>>) {
        self.entities.clear();
        self.positions.clear();
        self.masses.clear();

        for (entity, transform, mass) in query.iter() {
            if mass.value <= f32::EPSILON {
                continue;
            }
            self.entities.push(entity);
            self.positions.push(transform.translation);
            self.masses.push(mass.value);
        }
    }
}

/// Reusable buffers for the mutual gravity pair-once loop.
#[derive(Default)]
struct MutualGravityBuffers {
    bodies: Vec<GravityBody>,
    forces: Vec<Vec3>,
}

/// Plummer-softened gravitational force: F = G·m₁·m₂·r / (r²+ε²)^(3/2).
///
/// This is the proper gradient of the Plummer potential Φ = -G·M/√(r²+ε²),
/// giving F→0 as r→0 (physically correct softening). The combined formula
/// avoids a separate normalize call, saving one sqrt per pair (particular-style).
fn pair_force_vector(
    source_pos: Vec3,
    source_mass: f32,
    affected_pos: Vec3,
    affected_mass: f32,
    gravitational_constant: f32,
    softening_squared: f32,
) -> Option<Vec3> {
    let direction = source_pos - affected_pos;
    let distance_squared = direction.length_squared();
    if distance_squared <= f32::EPSILON {
        return None;
    }

    let norm_s = distance_squared + softening_squared;
    let force_scalar =
        gravitational_constant * source_mass * affected_mass / (norm_s * norm_s.sqrt());
    if !force_scalar.is_finite() {
        return None;
    }

    Some(direction * force_scalar)
}

// Barnes-Hut spatial partitioning
mod spatial {
    use bevy::prelude::*;

    #[derive(Clone, Debug)]
    pub struct AABB {
        pub center: Vec2,
        pub half_size: Vec2,
    }

    impl AABB {
        pub fn new(center: Vec2, half_size: Vec2) -> Self {
            Self { center, half_size }
        }

        pub fn contains(&self, point: Vec2) -> bool {
            let min = self.center - self.half_size;
            let max = self.center + self.half_size;
            point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
        }

        pub fn get_quadrant(&self, point: Vec2) -> usize {
            // Bit 0: right side (1) or left side (0)
            // Bit 1: bottom side (1) or top side (0)
            ((point.x >= self.center.x) as usize) | (((point.y < self.center.y) as usize) << 1)
        }

        pub fn get_quadrant_aabb(&self, quadrant: usize) -> AABB {
            let quarter_size = self.half_size * 0.5;
            let x_sign = if (quadrant & 1) == 0 { -1.0 } else { 1.0 };
            let y_sign = if (quadrant & 2) == 0 { 1.0 } else { -1.0 };

            AABB::new(
                self.center + Vec2::new(x_sign * quarter_size.x, y_sign * quarter_size.y),
                quarter_size,
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct MassProperties {
        pub total_mass: f32,
        pub center_of_mass: Vec3,
    }

    impl Default for MassProperties {
        fn default() -> Self {
            Self {
                total_mass: 0.0,
                center_of_mass: Vec3::ZERO,
            }
        }
    }

    impl MassProperties {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_body(&mut self, position: Vec3, mass: f32) {
            let new_total_mass = self.total_mass + mass;
            if new_total_mass > 0.0 {
                self.center_of_mass =
                    (self.center_of_mass * self.total_mass + position * mass) / new_total_mass;
                self.total_mass = new_total_mass;
            }
        }
    }

    #[derive(Debug)]
    pub struct QuadtreeNode {
        pub aabb: AABB,
        pub depth: usize,
        pub mass_properties: MassProperties,
        pub bodies: Vec<(Entity, Vec3, f32)>,
        pub children: [Option<Box<QuadtreeNode>>; 4],
        pub max_depth: usize,
        pub max_bodies_per_node: usize,
    }

    impl QuadtreeNode {
        pub fn new(aabb: AABB, depth: usize, max_depth: usize, max_bodies_per_node: usize) -> Self {
            Self {
                aabb,
                depth,
                mass_properties: MassProperties::new(),
                bodies: Vec::new(),
                children: [None, None, None, None],
                max_depth,
                max_bodies_per_node,
            }
        }

        pub fn is_far_enough(&self, position: Vec3, theta: f32) -> bool {
            let pos_2d = Vec2::new(position.x, position.y);
            let distance = (pos_2d - self.aabb.center).length();

            if distance < 0.001 || self.mass_properties.total_mass <= 0.0 {
                return false;
            }

            let width = self.aabb.half_size.x * 2.0;
            width / distance < theta
        }

        pub fn insert(&mut self, entity: Entity, position: Vec3, mass: f32) {
            self.mass_properties.add_body(position, mass);

            if self.depth >= self.max_depth
                || (self.bodies.len() < self.max_bodies_per_node && self.children[0].is_none())
            {
                self.bodies.push((entity, position, mass));
                return;
            }

            if self.children[0].is_none() {
                for i in 0..4 {
                    self.children[i] = Some(Box::new(QuadtreeNode::new(
                        self.aabb.get_quadrant_aabb(i),
                        self.depth + 1,
                        self.max_depth,
                        self.max_bodies_per_node,
                    )));
                }

                let existing_bodies = std::mem::take(&mut self.bodies);
                for (e, p, m) in existing_bodies {
                    let q = self.aabb.get_quadrant(p.truncate());
                    if let Some(child) = &mut self.children[q] {
                        child.insert(e, p, m);
                    }
                }
            }

            let quadrant = self.aabb.get_quadrant(position.truncate());
            if let Some(child) = &mut self.children[quadrant] {
                child.insert(entity, position, mass);
            }
        }
    }

    #[derive(Debug)]
    pub struct Quadtree {
        pub root: QuadtreeNode,
    }

    impl Quadtree {
        pub fn new(bounds: AABB, max_depth: usize, max_bodies_per_node: usize) -> Self {
            Self {
                root: QuadtreeNode::new(bounds, 0, max_depth, max_bodies_per_node),
            }
        }

        pub fn from_bodies(
            bodies: &[(Entity, Vec3, f32)],
            max_depth: usize,
            max_bodies_per_node: usize,
        ) -> Self {
            if bodies.is_empty() {
                return Self::new(
                    AABB::new(Vec2::ZERO, Vec2::new(1000.0, 1000.0)),
                    max_depth,
                    max_bodies_per_node,
                );
            }

            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;

            for (_, pos, _) in bodies {
                min_x = min_x.min(pos.x);
                min_y = min_y.min(pos.y);
                max_x = max_x.max(pos.x);
                max_y = max_y.max(pos.y);
            }

            let padding = ((max_x - min_x) + (max_y - min_y)) * 0.1;
            min_x -= padding;
            min_y -= padding;
            max_x += padding;
            max_y += padding;

            let center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
            let half_size = Vec2::new((max_x - min_x) * 0.5, (max_y - min_y) * 0.5);
            let max_half_size = half_size.x.max(half_size.y);

            let mut tree = Self::new(
                AABB::new(center, Vec2::splat(max_half_size)),
                max_depth,
                max_bodies_per_node,
            );

            for &(entity, position, mass) in bodies {
                tree.insert(entity, position, mass);
            }

            tree
        }

        pub fn insert(&mut self, entity: Entity, position: Vec3, mass: f32) {
            self.root.insert(entity, position, mass);
        }
    }
}

pub fn apply_uniform_gravity(
    gravity: Res<UniformGravity>,
    mut query: Query<(Entity, &Mass, &mut AppliedForce), With<GravityAffected>>,
    mut commands: Commands,
) {
    for (entity, mass, mut force) in &mut query {
        let gravity_force = mass.value * gravity.acceleration;
        force.force += gravity_force;

        if mass.value > 1000.0 {
            commands.entity(entity).insert(MassiveBody);
        }
    }
}

/// Compute mutual gravitational attraction for bodies that are gravity sources.
/// In this mode, every source both exerts and receives force.
#[allow(private_interfaces)]
pub fn calculate_mutual_gravitational_attraction(
    gravity_params: Res<GravityParams>,
    mut query: Query<(Entity, &Transform, &Mass, &mut AppliedForce), With<GravitySource>>,
    mut ctx: Local<MutualGravityBuffers>,
) {
    let softening_squared = gravity_params.softening * gravity_params.softening;
    let gravitational_constant = gravity_params.gravitational_constant;

    // Stage source data once (particular-style storage split).
    // Reuses heap allocations from previous frame via Local<>.
    ctx.bodies.clear();
    ctx.bodies.extend(
        query
            .iter()
            .map(|(entity, transform, mass, _)| GravityBody {
                entity,
                position: transform.translation,
                mass: mass.value,
            }),
    );
    // Stable ordering keeps accumulation deterministic for replay/debug.
    ctx.bodies.sort_by_key(|body| body.entity.to_bits());

    if ctx.bodies.len() > MUTUAL_REALTIME_BODY_LIMIT {
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            warn!(
                "Mutual gravity with {} bodies exceeds LP-0 realtime guideline ({}). Consider one-way gravity or Barnes-Hut.",
                ctx.bodies.len(),
                MUTUAL_REALTIME_BODY_LIMIT
            );
        }
    }

    // Zero the force accumulator (reuses allocation).
    let body_count = ctx.bodies.len();
    ctx.forces.clear();
    ctx.forces.resize(body_count, Vec3::ZERO);
    // TODO(LP-1): For mutual mode above MUTUAL_REALTIME_BODY_LIMIT, add an approximate
    // path (e.g. Barnes-Hut variant with symmetry corrections) to keep realtime budgets.

    // Pair-once pass, add opposite forces by index.
    for i in 0..ctx.bodies.len() {
        let body_a = ctx.bodies[i];
        for j in (i + 1)..ctx.bodies.len() {
            let body_b = ctx.bodies[j];
            let Some(force_on_a) = pair_force_vector(
                body_b.position,
                body_b.mass,
                body_a.position,
                body_a.mass,
                gravitational_constant,
                softening_squared,
            ) else {
                continue;
            };
            ctx.forces[i] += force_on_a;
            ctx.forces[j] -= force_on_a; // Newton's 3rd law
        }
    }

    // Single writeback pass to ECS.
    for (index, body) in ctx.bodies.iter().enumerate() {
        if let Ok((_, _, _, mut applied_force)) = query.get_mut(body.entity) {
            applied_force.force += ctx.forces[index];
        }
    }
}

pub fn calculate_gravitational_attraction(
    gravity_params: Res<GravityParams>,
    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
    mut affected_query: Query<
        (Entity, &Transform, &Mass, &mut AppliedForce),
        With<GravityAffected>,
    >,
) {
    let softening_squared = gravity_params.softening * gravity_params.softening;
    let gravitational_constant = gravity_params.gravitational_constant;

    // Particular-style storage split: stage read-only source data into tight SoA
    // buffers once, then iterate without ECS lookups in the hot loop.
    let mut sources = StagedGravitySources::default();
    sources.stage(&query);

    // Safe to parallelize: each worker mutates a distinct affected entity while sources stay read-only.
    one_way_gravity_inner(
        &sources,
        &mut affected_query,
        gravitational_constant,
        softening_squared,
    );
}

/// Core one-way gravity loop, extracted so Barnes-Hut can reuse it for small-N fallback.
fn one_way_gravity_inner(
    sources: &StagedGravitySources,
    affected_query: &mut Query<
        (Entity, &Transform, &Mass, &mut AppliedForce),
        With<GravityAffected>,
    >,
    gravitational_constant: f32,
    softening_squared: f32,
) {
    affected_query.par_iter_mut().for_each(
        |(affected_entity, affected_transform, affected_mass, mut force)| {
            let affected_pos = affected_transform.translation;

            for i in 0..sources.entities.len() {
                if sources.entities[i] == affected_entity {
                    continue;
                }

                let Some(force_vector) = pair_force_vector(
                    sources.positions[i],
                    sources.masses[i],
                    affected_pos,
                    affected_mass.value,
                    gravitational_constant,
                    softening_squared,
                ) else {
                    continue;
                };
                force.force += force_vector;
            }
        },
    );
}

pub fn calculate_barnes_hut_attraction(
    gravity_params: Res<GravityParams>,
    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
    mut affected_query: Query<
        (Entity, &Transform, &Mass, &mut AppliedForce),
        With<GravityAffected>,
    >,
    theta: f32, // Accuracy parameter (0.0-1.0, lower = more accurate)
) {
    let softening = gravity_params.softening;
    let softening_squared = softening * softening;
    let gravitational_constant = gravity_params.gravitational_constant;

    // Small-N fallback: brute force via extracted inner function (avoids quadtree overhead).
    if query.iter().count() < 20 {
        let mut sources = StagedGravitySources::default();
        sources.stage(&query);
        one_way_gravity_inner(
            &sources,
            &mut affected_query,
            gravitational_constant,
            softening_squared,
        );
        return;
    }

    // Stage directly into tuple vec for quadtree (no intermediate GravityBody).
    let body_data: Vec<_> = query
        .iter()
        .map(|(entity, transform, mass)| (entity, transform.translation, mass.value))
        .collect();
    let quadtree = spatial::Quadtree::from_bodies(
        &body_data,
        gravity_params.barnes_hut_max_depth,
        gravity_params.barnes_hut_max_bodies_per_node,
    );

    affected_query
        .par_iter_mut()
        .for_each(|(entity, transform, mass, mut force)| {
            let position = transform.translation;

            let force_vector = calculate_barnes_hut_force(
                entity,
                position,
                mass.value,
                &quadtree.root,
                theta,
                softening,
                gravitational_constant,
            );

            force.force += force_vector;
        });
}

pub fn calculate_barnes_hut_force(
    affected_entity: Entity,
    affected_position: Vec3,
    affected_mass: f32,
    node: &spatial::QuadtreeNode,
    theta: f32,
    softening: f32,
    gravitational_constant: f32,
) -> Vec3 {
    let softening_squared = softening * softening;

    if node.is_far_enough(affected_position, theta) {
        let direction = node.mass_properties.center_of_mass - affected_position;
        let distance_squared = direction.length_squared();
        let norm_s = distance_squared + softening_squared;
        // Plummer kernel: F = G·m·M·r / (r²+ε²)^(3/2)
        let force_scalar = gravitational_constant * affected_mass * node.mass_properties.total_mass
            / (norm_s * norm_s.sqrt());

        if !force_scalar.is_finite() {
            return Vec3::ZERO;
        }

        return direction * force_scalar;
    }

    if node.children.iter().all(|c| c.is_none()) {
        let mut total_force = Vec3::ZERO;

        for &(entity, position, mass) in &node.bodies {
            if entity == affected_entity {
                continue;
            }

            let direction = position - affected_position;
            let distance_squared = direction.length_squared();

            if distance_squared < 0.001 {
                continue;
            }

            let norm_s = distance_squared + softening_squared;
            let force_scalar =
                gravitational_constant * affected_mass * mass / (norm_s * norm_s.sqrt());

            if !force_scalar.is_finite() {
                continue;
            }

            total_force += direction * force_scalar;
        }

        return total_force;
    }

    let mut total_force = Vec3::ZERO;
    for child_node in node.children.iter().flatten() {
        total_force += calculate_barnes_hut_force(
            affected_entity,
            affected_position,
            affected_mass,
            child_node,
            theta,
            softening,
            gravitational_constant,
        );
    }

    total_force
}

// TODO: Gravitational potential energy U = -G*m1*m2/r is not tracked/softened.
// Forces are softened to avoid singularities, but potential energy is not recorded in the ledger.

/// Circular orbital velocity for pure Newtonian gravity (no softening).
/// v = sqrt(G·M / r)
pub fn calculate_orbital_velocity(central_mass: f32, orbit_radius: f32) -> f32 {
    (DEFAULT_GRAVITATIONAL_CONSTANT * central_mass / orbit_radius).sqrt()
}

/// Circular orbital velocity for Plummer-softened gravity.
/// Derived from F_centripetal = F_Plummer:
///   m·v²/r = G·M·m·r / (r²+ε²)^(3/2)
///   v = r · sqrt(G·M / (r²+ε²)^(3/2))
///
/// Converges to the unsoftened formula as softening → 0.
pub fn calculate_plummer_orbital_velocity(
    central_mass: f32,
    orbit_radius: f32,
    softening: f32,
) -> f32 {
    let norm_s = orbit_radius * orbit_radius + softening * softening;
    orbit_radius * (DEFAULT_GRAVITATIONAL_CONSTANT * central_mass / (norm_s * norm_s.sqrt())).sqrt()
}

pub fn calculate_elliptical_orbit_velocity(
    central_mass: f32,
    distance: f32,
    eccentricity: f32,
    is_periapsis: bool,
) -> f32 {
    let mu = DEFAULT_GRAVITATIONAL_CONSTANT * central_mass;
    let semimajor_axis = distance / (1.0 - eccentricity * if is_periapsis { 1.0 } else { -1.0 });
    (mu * (2.0 / distance - 1.0 / semimajor_axis)).sqrt()
}

pub fn calculate_escape_velocity(central_mass: f32, distance: f32) -> f32 {
    (2.0 * DEFAULT_GRAVITATIONAL_CONSTANT * central_mass / distance).sqrt()
}

#[derive(Default)]
pub struct GravityPlugin {
    /// Use Barnes-Hut optimization for n-body simulations
    pub use_barnes_hut: bool,
    /// Barnes-Hut accuracy parameter (lower is more accurate but slower)
    pub barnes_hut_theta: f32,
}

impl GravityPlugin {
    pub fn new() -> Self {
        Self {
            use_barnes_hut: true,
            barnes_hut_theta: 0.5,
        }
    }

    pub fn with_barnes_hut(mut self, enabled: bool) -> Self {
        self.use_barnes_hut = enabled;
        self
    }

    pub fn with_theta(mut self, theta: f32) -> Self {
        self.barnes_hut_theta = theta.clamp(0.1, 1.0);
        self
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GravitySet {
    /// Apply uniform gravity forces (like Earth's gravity)
    UniformGravity,
    /// Calculate n-body gravitational forces
    NBodyGravity,
}

fn use_mutual(mode: Res<GravityForceMode>) -> bool {
    *mode == GravityForceMode::Mutual
}

fn use_one_way(mode: Res<GravityForceMode>) -> bool {
    *mode == GravityForceMode::OneWay
}

fn has_many_sources(query: Query<(Entity, &Transform, &Mass), With<GravitySource>>) -> bool {
    query.iter().count() >= 20
}

fn has_few_sources(query: Query<(Entity, &Transform, &Mass), With<GravitySource>>) -> bool {
    query.iter().count() < 20
}

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GravityParams>()
            .init_resource::<UniformGravity>()
            .init_resource::<GravityForceMode>()
            .configure_sets(
                Update,
                (GravitySet::UniformGravity, GravitySet::NBodyGravity)
                    .chain()
                    .in_set(PhysicsSet::AccumulateForces),
            )
            .add_systems(
                Update,
                apply_uniform_gravity.in_set(GravitySet::UniformGravity),
            );

        app.add_systems(
            Update,
            calculate_mutual_gravitational_attraction
                .in_set(GravitySet::NBodyGravity)
                .run_if(use_mutual),
        );

        if self.use_barnes_hut {
            let theta = self.barnes_hut_theta;

            app.add_systems(
                Update,
                (move |gravity_params: Res<GravityParams>,
                       query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
                       affected_query: Query<
                    (Entity, &Transform, &Mass, &mut AppliedForce),
                    With<GravityAffected>,
                >| {
                    calculate_barnes_hut_attraction(gravity_params, query, affected_query, theta);
                })
                .in_set(GravitySet::NBodyGravity)
                .run_if(use_one_way)
                .run_if(has_many_sources),
            );

            app.add_systems(
                Update,
                calculate_gravitational_attraction
                    .in_set(GravitySet::NBodyGravity)
                    .run_if(use_one_way)
                    .run_if(has_few_sources),
            );
        } else {
            app.add_systems(
                Update,
                calculate_gravitational_attraction
                    .in_set(GravitySet::NBodyGravity)
                    .run_if(use_one_way),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_body_action_reaction() {
        // Test Newton's 3rd Law: two-body gravity should produce equal/opposite forces
        let params = GravityParams::default();
        let g = params.gravitational_constant;
        let softening_sq = params.softening * params.softening;

        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(10.0, 0.0, 0.0);
        let mass1 = 5.0;
        let mass2 = 3.0;

        // Plummer kernel: F = G·m1·m2·r / (r²+ε²)^(3/2)
        let direction = pos2 - pos1;
        let dist_sq = direction.length_squared();
        let norm_s = dist_sq + softening_sq;
        let force_scalar = g * mass1 * mass2 / (norm_s * norm_s.sqrt());

        let force_on_2 = direction * force_scalar;
        let force_on_1 = -force_on_2;

        // Verify equal magnitude, opposite direction
        assert!((force_on_1.length() - force_on_2.length()).abs() < 1e-5);
        assert!(
            (force_on_1 + force_on_2).length() < 1e-5,
            "Action-reaction forces do not cancel"
        );
    }

    #[test]
    fn test_barnes_hut_vs_brute_force_small_n() {
        // For small N, Barnes-Hut should match brute force closely
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(GravityParams::default());

        // Create 3 bodies
        let bodies = vec![
            (Vec3::new(0.0, 0.0, 0.0), 100.0),
            (Vec3::new(50.0, 0.0, 0.0), 50.0),
            (Vec3::new(0.0, 50.0, 0.0), 50.0),
        ];

        let entities: Vec<_> = bodies
            .iter()
            .map(|(pos, mass)| {
                app.world_mut()
                    .spawn((
                        Transform::from_translation(*pos),
                        Mass::new(*mass),
                        AppliedForce::new(Vec3::ZERO),
                        GravitySource,
                        GravityAffected,
                    ))
                    .id()
            })
            .collect();

        // Compute brute force
        let mut brute_forces = Vec::new();
        for _ in &entities {
            brute_forces.push(Vec3::ZERO);
        }

        let params = app.world().resource::<GravityParams>();
        let g = params.gravitational_constant;
        let soft_sq = params.softening * params.softening;

        for i in 0..entities.len() {
            for j in 0..entities.len() {
                if i == j {
                    continue;
                }
                let pos_i = bodies[i].0;
                let pos_j = bodies[j].0;
                let mass_j = bodies[j].1;
                let dir = pos_j - pos_i;
                let dist_sq = dir.length_squared();
                // Plummer kernel: F = G·m1·m2·r / (r²+ε²)^(3/2)
                let norm_s = dist_sq + soft_sq;
                let force_scalar = g * mass_j * bodies[i].1 / (norm_s * norm_s.sqrt());
                brute_forces[i] += dir * force_scalar;
            }
        }

        // Build quadtree and compute BH force
        let body_data: Vec<_> = entities
            .iter()
            .enumerate()
            .map(|(i, &e)| (e, bodies[i].0, bodies[i].1))
            .collect();
        let quadtree = spatial::Quadtree::from_bodies(
            &body_data,
            params.barnes_hut_max_depth,
            params.barnes_hut_max_bodies_per_node,
        );

        for (i, &entity) in entities.iter().enumerate() {
            let bh_force = calculate_barnes_hut_force(
                entity,
                bodies[i].0,
                bodies[i].1,
                &quadtree.root,
                0.5,
                params.softening,
                g,
            );
            let diff = (bh_force - brute_forces[i]).length();
            assert!(diff < 1.0, "BH force differs from brute force by {}", diff);
        }
    }
}
