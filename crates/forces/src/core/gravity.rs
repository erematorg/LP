use super::newton_laws::{AppliedForce, Mass};
use bevy::prelude::*;

// Simulation constants
pub const DEFAULT_GRAVITATIONAL_CONSTANT: f32 = 0.1;

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

    let sources: Vec<(Entity, Vec3, f32)> = query
        .iter()
        .map(|(e, t, m)| (e, t.translation, m.value))
        .collect();

    affected_query.par_iter_mut().for_each(
        |(affected_entity, affected_transform, affected_mass, mut force)| {
            let affected_pos = affected_transform.translation;

            for &(source_entity, source_pos, source_mass) in &sources {
                if source_entity == affected_entity {
                    continue;
                }

                let direction = source_pos - affected_pos;
                let distance_squared = direction.length_squared();
                let softened_distance_squared = distance_squared + softening_squared;
                let force_magnitude = gravitational_constant * source_mass * affected_mass.value
                    / softened_distance_squared;
                force.force += direction.normalize() * force_magnitude;
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
    // Only use Barnes-Hut for larger simulations
    if query.iter().count() < 20 {
        calculate_gravitational_attraction(gravity_params, query, affected_query);
        return;
    }

    let bodies: Vec<(Entity, Vec3, f32)> = query
        .iter()
        .map(|(e, t, m)| (e, t.translation, m.value))
        .collect();

    let quadtree = spatial::Quadtree::from_bodies(
        &bodies,
        gravity_params.barnes_hut_max_depth,
        gravity_params.barnes_hut_max_bodies_per_node,
    );
    let softening = gravity_params.softening;
    let gravitational_constant = gravity_params.gravitational_constant;

    affected_query
        .par_iter_mut()
        .for_each(|(entity, transform, _, mut force)| {
            let position = transform.translation;

            if bodies.iter().any(|&(e, _, _)| e == entity) {
                return;
            }

            let force_vector = calculate_barnes_hut_force(
                position,
                &quadtree.root,
                theta,
                softening,
                gravitational_constant,
            );

            force.force += force_vector;
        });
}

pub fn calculate_barnes_hut_force(
    affected_position: Vec3,
    node: &spatial::QuadtreeNode,
    theta: f32,
    softening: f32,
    gravitational_constant: f32,
) -> Vec3 {
    let softening_squared = softening * softening;

    if node.is_far_enough(affected_position, theta) {
        let direction = node.mass_properties.center_of_mass - affected_position;
        let distance_squared = direction.length_squared();
        let softened_distance_squared = distance_squared + softening_squared;
        let force_magnitude =
            gravitational_constant * node.mass_properties.total_mass / softened_distance_squared;
        return direction.normalize() * force_magnitude;
    }

    if node.children.iter().all(|c| c.is_none()) {
        let mut total_force = Vec3::ZERO;

        for &(_, position, mass) in &node.bodies {
            let direction = position - affected_position;
            let distance_squared = direction.length_squared();

            if distance_squared < 0.001 {
                continue;
            }

            let softened_distance_squared = distance_squared + softening_squared;
            let force_magnitude = gravitational_constant * mass / softened_distance_squared;
            total_force += direction.normalize() * force_magnitude;
        }

        return total_force;
    }

    let mut total_force = Vec3::ZERO;
    for child_node in node.children.iter().flatten() {
        total_force += calculate_barnes_hut_force(
            affected_position,
            child_node,
            theta,
            softening,
            gravitational_constant,
        );
    }

    total_force
}

pub fn calculate_orbital_velocity(central_mass: f32, orbit_radius: f32) -> f32 {
    (DEFAULT_GRAVITATIONAL_CONSTANT * central_mass / orbit_radius).sqrt()
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

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GravityParams>()
            .init_resource::<UniformGravity>()
            .configure_sets(
                Update,
                (GravitySet::UniformGravity, GravitySet::NBodyGravity).chain(),
            )
            .add_systems(
                Update,
                apply_uniform_gravity.in_set(GravitySet::UniformGravity),
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
                .run_if(
                    |query: Query<(Entity, &Transform, &Mass), With<GravitySource>>| {
                        query.iter().count() >= 20
                    },
                ),
            );

            app.add_systems(
                Update,
                calculate_gravitational_attraction
                    .in_set(GravitySet::NBodyGravity)
                    .run_if(
                        |query: Query<(Entity, &Transform, &Mass), With<GravitySource>>| {
                            query.iter().count() < 20
                        },
                    ),
            );
        } else {
            app.add_systems(
                Update,
                calculate_gravitational_attraction.in_set(GravitySet::NBodyGravity),
            );
        }
    }
}
