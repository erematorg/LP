use bevy::prelude::*;
use crate::mechanics::{AppliedForce, Mass};

/// Modified gravitational constant for simulation scale
pub const GRAVITATIONAL_CONSTANT: f32 = 0.1;

/// Component for uniform gravitational field (like on Earth's surface)
#[derive(Resource, Debug, Clone, Copy)]
pub struct UniformGravity {
    pub acceleration: Vec3,
}

impl Default for UniformGravity {
    fn default() -> Self {
        Self { acceleration: Vec3::new(0.0, -9.81, 0.0) }
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
    use bevy::prelude::*;
    
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
            point.x >= min.x && point.x <= max.x && 
            point.y >= min.y && point.y <= max.y
        }
        
        pub fn get_quadrant(&self, point: Vec2) -> usize {
            let is_right = point.x >= self.center.x;
            let is_bottom = point.y < self.center.y;
            
            match (is_right, is_bottom) {
                (false, false) => 0, // top-left
                (true, false) => 1,  // top-right
                (false, true) => 2,  // bottom-left
                (true, true) => 3,   // bottom-right
            }
        }
        
        pub fn get_quadrant_aabb(&self, quadrant: usize) -> AABB {
            let quarter_size = self.half_size * 0.5;
            let offset = match quadrant {
                0 => Vec2::new(-quarter_size.x, quarter_size.y),  // top-left
                1 => Vec2::new(quarter_size.x, quarter_size.y),   // top-right
                2 => Vec2::new(-quarter_size.x, -quarter_size.y), // bottom-left
                3 => Vec2::new(quarter_size.x, -quarter_size.y),  // bottom-right
                _ => panic!("Invalid quadrant"),
            };
            
            AABB::new(self.center + offset, quarter_size)
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
            Self { total_mass: 0.0, center_of_mass: Vec3::ZERO }
        }
        
        pub fn add_body(&mut self, position: Vec3, mass: f32) {
            let new_total_mass = self.total_mass + mass;
            if new_total_mass > 0.0 {
                self.center_of_mass = (self.center_of_mass * self.total_mass + position * mass) / new_total_mass;
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
            
            if self.depth >= MAX_DEPTH || (self.bodies.len() < MAX_BODIES_PER_NODE && self.children[0].is_none()) {
                self.bodies.push((entity, position, mass));
                return;
            }
            
            if self.children[0].is_none() {
                for i in 0..4 {
                    self.children[i] = Some(Box::new(QuadtreeNode::new(
                        self.aabb.get_quadrant_aabb(i), 
                        self.depth + 1
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
            Self { root: QuadtreeNode::new(bounds, 0) }
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
    for (entity, mass, mut force) in query.iter_mut() {
        let gravity_force = mass.value * gravity.acceleration;
        force.force += gravity_force;
        
        if mass.value > 1000.0 {
            commands.entity(entity).insert(MassiveBody);
        }
    }
}

/// System to calculate gravitational attraction between entities
pub fn calculate_gravitational_attraction(
    query: Query<(Entity, &Transform, &Mass), With<GravitySource>>,
    mut affected_query: Query<(Entity, &Transform, &Mass, &mut AppliedForce), With<GravityAffected>>,
) {
    // Determine if we should use SIMD optimization
    let use_simd = query.iter().count() >= 8 && affected_query.iter().count() >= 8;
    
    if use_simd {
        // Collect sources for batch processing
        let sources: Vec<(Entity, Vec3, f32)> = query
            .iter()
            .map(|(e, t, m)| (e, t.translation, m.value))
            .collect();

        // Process in batches of 4 for better SIMD optimization
        for chunk in sources.chunks(4) {
            for (affected_entity, affected_transform, affected_mass, mut force) in affected_query.iter_mut() {
                let affected_pos = affected_transform.translation;
                
                for &(source_entity, source_pos, source_mass) in chunk {
                    if source_entity == affected_entity {
                        continue;
                    }
                    
                    let direction = source_pos - affected_pos;
                    let distance_squared = direction.length_squared();
                    let safe_distance_squared = distance_squared.max(25.0);
                    let force_magnitude = GRAVITATIONAL_CONSTANT * 
                        (source_mass * affected_mass.value) / safe_distance_squared;
                    
                    force.force += direction.normalize() * force_magnitude;
                }
            }
        }
    } else {
        // Use original non-SIMD implementation for small body counts
        for (source_entity, source_transform, source_mass) in query.iter() {
            for (affected_entity, affected_transform, affected_mass, mut force) in affected_query.iter_mut() {
                if source_entity == affected_entity {
                    continue;
                }
                
                let direction = source_transform.translation - affected_transform.translation;
                let distance_squared = direction.length_squared();
                let safe_distance_squared = distance_squared.max(25.0);
                let force_magnitude = GRAVITATIONAL_CONSTANT * 
                    (source_mass.value * affected_mass.value) / safe_distance_squared;
                
                force.force += direction.normalize() * force_magnitude;
            }
        }
    }
}

/// Calculate gravitational force using the Barnes-Hut approximation method
pub fn calculate_barnes_hut_force(
    affected_position: Vec3, 
    node: &spatial::QuadtreeNode,
    theta: f32
) -> Vec3 {
    // If the node is far enough, use approximation
    if node.is_far_enough(affected_position, theta) {
        let direction = node.mass_properties.center_of_mass - affected_position;
        let distance_squared = direction.length_squared().max(0.001);
        let force_magnitude = GRAVITATIONAL_CONSTANT * node.mass_properties.total_mass / distance_squared;
        return direction.normalize() * force_magnitude;
    }
    
    // If leaf node, calculate force from each body
    if node.children.iter().all(|c| c.is_none()) {
        let mut total_force = Vec3::ZERO;
        
        for &(_, position, mass) in &node.bodies {
            let direction = position - affected_position;
            let distance_squared = direction.length_squared();
            
            if distance_squared < 0.001 {
                continue;
            }
            
            let force_magnitude = GRAVITATIONAL_CONSTANT * mass / distance_squared;
            total_force += direction.normalize() * force_magnitude;
        }
        
        return total_force;
    }
    
    // Sum forces from children
    let mut total_force = Vec3::ZERO;
    for child in &node.children {
        if let Some(child_node) = child {
            total_force += calculate_barnes_hut_force(affected_position, child_node, theta);
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