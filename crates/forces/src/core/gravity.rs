use super::newton_laws::{AppliedForce, Mass};
use bevy::prelude::*;

/// A trait for computing the squared norm of a vector efficiently
trait Norm {
    type Output;
    fn norm_squared(self) -> Self::Output;
}

impl Norm for Vec3 {
    type Output = f32;
    #[inline]
    fn norm_squared(self) -> f32 {
        self.length_squared()
    }
}

/// Modified gravitational constant for simulation scale
pub const GRAVITATIONAL_CONSTANT: f32 = 0.1;

/// Calculate softened gravitational acceleration between two bodies
#[inline]
fn calculate_softened_acceleration(direction: Vec3, mass: f32, softening_squared: f32) -> Vec3 {
    let distance_squared = direction.norm_squared();
    // Apply softening to prevent singularities
    let softened_distance_squared = distance_squared + softening_squared;
    let force_magnitude = GRAVITATIONAL_CONSTANT * mass / softened_distance_squared;

    direction.normalize() * force_magnitude
}

/// Resource for gravity simulation parameters
#[derive(Resource, Clone, Debug)]
pub struct GravityParams {
    /// Softening parameter to prevent singularities
    pub softening: f32,
}

impl Default for GravityParams {
    fn default() -> Self {
        Self { softening: 5.0 }
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
#[derive(Component, Debug, Clone, Copy)]
pub struct GravityAffected;

/// Marker component for gravity field measurement points
#[derive(Component, Debug, Clone, Copy)]
pub struct GravityFieldMarker;

/// Component for objects that generate gravitational attraction
#[derive(Component, Debug, Clone, Copy)]
pub struct GravitySource;

/// Marker for bodies with significant mass
#[derive(Component, Debug, Clone, Copy)]
pub struct MassiveBody;

// Spatial partitioning structures for Barnes-Hut algorithm
mod spatial {
    use bevy::prelude::*; //TODO: Redundant and may need to make this a clearer section instead

    const MAX_DEPTH: usize = 8;
    const MAX_BODIES_PER_NODE: usize = 8;

    /// 2D axis-aligned bounding box
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
            // x-offset: negative for left (quadrants 0,2), positive for right (quadrants 1,3)
            // y-offset: positive for top (quadrants 0,1), negative for bottom (quadrants 2,3)
            let x_sign = if (quadrant & 1) == 0 { -1.0 } else { 1.0 };
            let y_sign = if (quadrant & 2) == 0 { 1.0 } else { -1.0 };

            AABB::new(
                self.center + Vec2::new(x_sign * quarter_size.x, y_sign * quarter_size.y),
                quarter_size,
            )
        }
    }

    /// Mass properties for Barnes-Hut approximation
    #[derive(Clone, Debug)]
    pub struct MassProperties {
        pub total_mass: f32,
        pub center_of_mass: Vec3,
    }

    impl MassProperties {
        pub fn new() -> Self {
            Self {
                total_mass: 0.0,
                center_of_mass: Vec3::ZERO,
            }
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

    /// Node in the quadtree
    #[derive(Debug)]
    pub struct QuadtreeNode {
        pub aabb: AABB,
        pub depth: usize,
        pub mass_properties: MassProperties,
        pub bodies: Vec<(Entity, Vec3, f32)>, // Entity, position, mass
        pub children: [Option<Box<QuadtreeNode>>; 4],
    }

    impl QuadtreeNode {
        pub fn new(aabb: AABB, depth: usize) -> Self {
            Self {
                aabb,
                depth,
                mass_properties: MassProperties::new(),
                bodies: Vec::new(),
                children: [None, None, None, None],
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
            let pos_2d = Vec2::new(position.x, position.y);
            self.mass_properties.add_body(position, mass);

            if self.depth >= MAX_DEPTH
                || (self.bodies.len() < MAX_BODIES_PER_NODE && self.children[0].is_none())
            {
                self.bodies.push((entity, position, mass));
                return;
            }

            if self.children[0].is_none() {
                for i in 0..4 {
                    self.children[i] = Some(Box::new(QuadtreeNode::new(
                        self.aabb.get_quadrant_aabb(i),
                        self.depth + 1,
                    )));
                }

                let existing_bodies = std::mem::take(&mut self.bodies);
                for (e, p, m) in existing_bodies {
                    let q = self.aabb.get_quadrant(Vec2::new(p.x, p.y));
                    if let Some(child) = &mut self.children[q] {
                        child.insert(e, p, m);
                    }
                }
            }

            let quadrant = self.aabb.get_quadrant(pos_2d);
            if let Some(child) = &mut self.children[quadrant] {
                child.insert(entity, position, mass);
            }
        }
    }

    /// Main quadtree structure
    #[derive(Debug)]
    pub struct Quadtree {
        pub root: QuadtreeNode,
    }

    impl Quadtree {
        pub fn new(bounds: AABB) -> Self {
            Self {
                root: QuadtreeNode::new(bounds, 0),
            }
        }

        pub fn from_bodies(bodies: &[(Entity, Vec3, f32)]) -> Self {
            if bodies.is_empty() {
                return Self::new(AABB::new(Vec2::ZERO, Vec2::new(1000.0, 1000.0)));
            }

            // Find bounds
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

            // Add padding and make square
            let padding = ((max_x - min_x) + (max_y - min_y)) * 0.1;
            min_x -= padding;
            min_y -= padding;
            max_x += padding;
            max_y += padding;

            let center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
            let half_size = Vec2::new((max_x - min_x) * 0.5, (max_y - min_y) * 0.5);
            let max_half_size = half_size.x.max(half_size.y);

            let mut tree = Self::new(AABB::new(center, Vec2::splat(max_half_size)));

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

/// System to apply uniform gravity forces to entities
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

/// System to calculate gravitational attraction between entities
pub fn calculate_gravitational_attraction(
    gravity_params: Res<GravityParams>,
    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
    mut affected_query: Query<
        (Entity, &Transform, &Mass, &mut AppliedForce),
        With<GravityAffected>,
    >,
) {
    // Get softening parameter squared once
    let softening_squared = gravity_params.softening * gravity_params.softening;

    // Collect all gravity sources once
    let sources: Vec<(Entity, Vec3, f32)> = query
        .iter()
        .map(|(e, t, m)| (e, t.translation, m.value))
        .collect();

    // Using Bevy's built-in parallelization - processes all affected entities in parallel
    affected_query.par_iter_mut().for_each(|(affected_entity, affected_transform, affected_mass, mut force)| {
        let affected_pos = affected_transform.translation;
        
        // Calculate the force from all sources on this affected entity
        for &(source_entity, source_pos, source_mass) in &sources {
            if source_entity == affected_entity {
                continue;
            }
            
            let direction = source_pos - affected_pos;
            force.force += calculate_softened_acceleration(
                direction,
                source_mass * affected_mass.value,
                softening_squared,
            );
        }
    });
}

/// System to calculate gravitational attraction using Barnes-Hut algorithm
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

    // Create quadtree from gravity sources
    let bodies: Vec<(Entity, Vec3, f32)> = query
        .iter()
        .map(|(e, t, m)| (e, t.translation, m.value))
        .collect();

    let quadtree = spatial::Quadtree::from_bodies(&bodies);

    // Using Bevy's built-in parallelization for the affected bodies
    affected_query.par_iter_mut().for_each(|(entity, transform, _, mut force)| {
        let position = transform.translation;
        
        // Skip self-attraction by checking if this entity is in the tree
        if bodies.iter().any(|&(e, _, _)| e == entity) {
            return; // Skip this iteration
        }
        
        // Calculate force using Barnes-Hut algorithm
        let force_vector = calculate_barnes_hut_force(
            position, 
            &quadtree.root, 
            theta, 
            gravity_params.softening
        );
        
        force.force += force_vector;
    });
}

/// Calculate gravitational force using the Barnes-Hut approximation method
pub fn calculate_barnes_hut_force(
    affected_position: Vec3,
    node: &spatial::QuadtreeNode,
    theta: f32,
    softening: f32,
) -> Vec3 {
    // Get softening squared once
    let softening_squared = softening * softening;

    // If the node is far enough, use approximation
    if node.is_far_enough(affected_position, theta) {
        let direction = node.mass_properties.center_of_mass - affected_position;
        return calculate_softened_acceleration(
            direction,
            node.mass_properties.total_mass,
            softening_squared,
        );
    }

    // If leaf node, calculate force from each body
    if node.children.iter().all(|c| c.is_none()) {
        let mut total_force = Vec3::ZERO;

        for &(_, position, mass) in &node.bodies {
            let direction = position - affected_position;
            let distance_squared = direction.norm_squared();

            if distance_squared < 0.001 {
                continue;
            }

            total_force += calculate_softened_acceleration(direction, mass, softening_squared);
        }

        return total_force;
    }

    // Sum forces from children
    let mut total_force = Vec3::ZERO;
    for child in &node.children {
        if let Some(child_node) = child {
            total_force +=
                calculate_barnes_hut_force(affected_position, child_node, theta, softening);
        }
    }

    total_force
}

/// Calculate orbital velocity for circular orbit
pub fn calculate_orbital_velocity(central_mass: f32, orbit_radius: f32) -> f32 {
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
    (mu * (2.0 / distance - 1.0 / semimajor_axis)).sqrt()
}

/// Calculate escape velocity from a massive body
pub fn calculate_escape_velocity(central_mass: f32, distance: f32) -> f32 {
    (2.0 * GRAVITATIONAL_CONSTANT * central_mass / distance).sqrt()
}

/// Plugin for gravity systems
#[derive(Default)]
pub struct GravityPlugin {
    /// Use Barnes-Hut optimization for n-body simulations
    pub use_barnes_hut: bool,
    /// Barnes-Hut accuracy parameter (lower is more accurate but slower)
    pub barnes_hut_theta: f32,
}

impl GravityPlugin {
    /// Create new gravity plugin with default settings
    pub fn new() -> Self {
        Self {
            use_barnes_hut: true,
            barnes_hut_theta: 0.5,
        }
    }
    
    /// Configure whether to use Barnes-Hut optimization
    pub fn with_barnes_hut(mut self, enabled: bool) -> Self {
        self.use_barnes_hut = enabled;
        self
    }
    
    /// Set the Barnes-Hut theta parameter
    pub fn with_theta(mut self, theta: f32) -> Self {
        self.barnes_hut_theta = theta.clamp(0.1, 1.0);
        self
    }
}

/// System set for gravity calculations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GravitySet {
    /// Apply uniform gravity forces (like Earth's gravity)
    UniformGravity,
    /// Calculate n-body gravitational forces
    NBodyGravity,
}

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register default resources if not present
            .init_resource::<GravityParams>()
            .init_resource::<UniformGravity>()
            
            // Configure gravity system sets
            .configure_sets(
                Update,
                (GravitySet::UniformGravity, GravitySet::NBodyGravity).chain()
            )
            
            // Add gravity systems
            .add_systems(
                Update,
                apply_uniform_gravity
                    .in_set(GravitySet::UniformGravity)
            );
            
        // Add n-body gravity systems with run conditions
        if self.use_barnes_hut {
            let theta = self.barnes_hut_theta;
            
            app.add_systems(
                Update,
                // Use a closure to pass the theta parameter
                (move |
                    gravity_params: Res<GravityParams>,
                    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
                    affected_query: Query<(Entity, &Transform, &Mass, &mut AppliedForce), With<GravityAffected>>,
                | {
                    calculate_barnes_hut_attraction(gravity_params, query, affected_query, theta);
                })
                .in_set(GravitySet::NBodyGravity)
                // Use run condition to only use Barnes-Hut for larger simulations
                .run_if(|query: Query<(Entity, &Transform, &Mass), With<GravitySource>>| {
                    query.iter().count() >= 20
                })
            );
            
            // Fallback to simple n-body for small simulations
            app.add_systems(
                Update,
                calculate_gravitational_attraction
                    .in_set(GravitySet::NBodyGravity)
                    .run_if(|query: Query<(Entity, &Transform, &Mass), With<GravitySource>>| {
                        query.iter().count() < 20
                    })
            );
        } else {
            // Always use simple n-body calculations if Barnes-Hut is disabled
            app.add_systems(
                Update,
                calculate_gravitational_attraction
                    .in_set(GravitySet::NBodyGravity)
            );
        }
    }
}