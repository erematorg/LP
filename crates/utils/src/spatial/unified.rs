//! Unified spatial indexing for physics participants only.
//!
//! - No crate cycles: this module knows nothing about Charge/Temperature/Mass
//! - Entities opt-in via `SpatiallyIndexed` marker (physics crates inject)
//! - Membership tracking: entity->position map is authoritative

use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::grid::SpatialGrid;

/// System sets for spatial index maintenance.
///
/// **Execution order**: InjectMarkers -> Maintain (in PreUpdate)
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SpatialIndexSet {
    /// Marker injection (physics crates insert SpatiallyIndexed)
    InjectMarkers,
    /// Index maintenance (attach/update/remove entities)
    Maintain,
}

/// Marker: entity participates in physics spatial queries.
///
/// Inserted by physics crates (energy/forces), never by utils.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct SpatiallyIndexed;

/// Cached last-known cell for change detection.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct SpatialCell {
    pub cell: (i32, i32),
}

/// Neighbor-search backend mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum NeighborSearchMode {
    /// Uniform cell field broadphase (hash grid).
    UniformCellField,
    /// LP-native tree index (AABB tree with bulk rebuild from tracked points).
    HierarchicalEnvelopeField,
    /// Runtime backend selection from observed sparsity.
    Adaptive,
}

/// Runtime configuration for unified spatial indexing.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct NeighborSearchConfig {
    /// Default cell size in meters (used for grid indexing and density estimation).
    pub cell_size_meters: f32,
    /// Backend selection policy.
    pub mode: NeighborSearchMode,
    /// Minimum entity count before Adaptive mode considers switching to tree mode.
    pub adaptive_min_entities_for_hierarchy: usize,
    /// If entities/cell is lower than this threshold, Adaptive mode prefers tree mode.
    pub adaptive_sparse_entities_per_cell_threshold: f32,
    /// Cooldown to avoid backend thrashing in Adaptive mode.
    pub adaptive_switch_cooldown_frames: u32,
    /// Maximum points per tree leaf.
    pub hierarchy_leaf_capacity: usize,
}

impl Default for NeighborSearchConfig {
    fn default() -> Self {
        Self {
            cell_size_meters: 50.0,
            mode: NeighborSearchMode::UniformCellField,
            adaptive_min_entities_for_hierarchy: 1000,
            adaptive_sparse_entities_per_cell_threshold: 0.35,
            adaptive_switch_cooldown_frames: 120,
            hierarchy_leaf_capacity: 24,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum BackendStorage {
    #[default]
    Grid,
    Tree,
}

#[derive(Clone, Copy, Debug)]
struct TreeAabb {
    min: Vec2,
    max: Vec2,
}

impl TreeAabb {
    fn from_points(points: &[(Entity, Vec2)]) -> Self {
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for (_, p) in points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Self { min, max }
    }

    fn merge(a: Self, b: Self) -> Self {
        Self {
            min: Vec2::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y)),
            max: Vec2::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y)),
        }
    }

    fn distance2_to_point(self, p: Vec2) -> f32 {
        let dx = if p.x < self.min.x {
            self.min.x - p.x
        } else if p.x > self.max.x {
            p.x - self.max.x
        } else {
            0.0
        };
        let dy = if p.y < self.min.y {
            self.min.y - p.y
        } else if p.y > self.max.y {
            p.y - self.max.y
        } else {
            0.0
        };
        dx * dx + dy * dy
    }

    fn widest_axis(self) -> usize {
        let ext = self.max - self.min;
        if ext.x >= ext.y { 0 } else { 1 }
    }
}

#[derive(Clone, Debug)]
enum TreeNode {
    Leaf {
        aabb: TreeAabb,
        points: Vec<(Entity, Vec2)>,
    },
    Branch {
        aabb: TreeAabb,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

impl TreeNode {
    fn aabb(&self) -> TreeAabb {
        match self {
            TreeNode::Leaf { aabb, .. } | TreeNode::Branch { aabb, .. } => *aabb,
        }
    }

    fn for_each_in_radius(&self, center: Vec2, radius2: f32, emit: &mut impl FnMut(Entity)) {
        if self.aabb().distance2_to_point(center) > radius2 {
            return;
        }

        match self {
            TreeNode::Leaf { points, .. } => {
                for (entity, p) in points {
                    if p.distance_squared(center) <= radius2 {
                        emit(*entity);
                    }
                }
            }
            TreeNode::Branch { left, right, .. } => {
                left.for_each_in_radius(center, radius2, emit);
                right.for_each_in_radius(center, radius2, emit);
            }
        }
    }
}

#[derive(Default)]
struct SpatialTreeIndex {
    root: Option<Box<TreeNode>>,
    leaf_size: usize,
}

impl SpatialTreeIndex {
    fn new(leaf_size: usize) -> Self {
        Self {
            root: None,
            leaf_size: leaf_size.max(4),
        }
    }

    fn set_leaf_size(&mut self, leaf_size: usize) {
        self.leaf_size = leaf_size.max(4);
    }

    fn rebuild_from_positions(&mut self, positions: &HashMap<Entity, Vec2>) {
        if positions.is_empty() {
            self.root = None;
            return;
        }

        let mut points: Vec<(Entity, Vec2)> = positions.iter().map(|(e, p)| (*e, *p)).collect();
        self.root = Some(Self::build_node(&mut points, self.leaf_size));
    }

    fn build_node(points: &mut Vec<(Entity, Vec2)>, leaf_size: usize) -> Box<TreeNode> {
        let aabb = TreeAabb::from_points(points);
        if points.len() <= leaf_size {
            return Box::new(TreeNode::Leaf {
                aabb,
                points: std::mem::take(points),
            });
        }

        let axis = aabb.widest_axis();
        points.sort_by(|a, b| {
            let va = if axis == 0 { a.1.x } else { a.1.y };
            let vb = if axis == 0 { b.1.x } else { b.1.y };
            match va.partial_cmp(&vb).unwrap_or(Ordering::Equal) {
                Ordering::Equal => a.0.to_bits().cmp(&b.0.to_bits()),
                ord => ord,
            }
        });

        let mid = points.len() / 2;
        let mut right_points = points.split_off(mid);
        let mut left_points = std::mem::take(points);

        let left = Self::build_node(&mut left_points, leaf_size);
        let right = Self::build_node(&mut right_points, leaf_size);
        let branch_aabb = TreeAabb::merge(left.aabb(), right.aabb());

        Box::new(TreeNode::Branch {
            aabb: branch_aabb,
            left,
            right,
        })
    }

    fn for_each_in_radius(&self, center: Vec2, radius: f32, mut emit: impl FnMut(Entity)) {
        if let Some(root) = &self.root {
            root.for_each_in_radius(center, radius * radius, &mut emit);
        }
    }
}

/// Unified spatial index with correct membership tracking.
///
/// **Correctness guarantees**:
/// - On insert/update: entity has exactly one stored position
/// - Query returns candidate entities only; callers apply exact physical filtering
/// - Backend switching preserves all tracked entities
#[derive(Resource)]
pub struct UnifiedSpatialIndex {
    grid: SpatialGrid,
    tree: SpatialTreeIndex,
    backend: BackendStorage,
    entity_positions: HashMap<Entity, Vec2>,
    config: NeighborSearchConfig,
    frames_since_switch: u32,
    tree_dirty: bool,
}

impl UnifiedSpatialIndex {
    pub fn new(cell_size_meters: f32) -> Self {
        let config = NeighborSearchConfig {
            cell_size_meters,
            ..NeighborSearchConfig::default()
        };
        Self::from_config(config)
    }

    pub fn from_config(config: NeighborSearchConfig) -> Self {
        Self {
            grid: SpatialGrid::new(config.cell_size_meters),
            tree: SpatialTreeIndex::new(config.hierarchy_leaf_capacity),
            backend: BackendStorage::Grid,
            entity_positions: HashMap::new(),
            config,
            frames_since_switch: 0,
            tree_dirty: false,
        }
    }

    pub fn set_config(&mut self, config: &NeighborSearchConfig) {
        let cell_size_changed =
            (self.config.cell_size_meters - config.cell_size_meters).abs() > f32::EPSILON;
        self.config = config.clone();
        self.tree.set_leaf_size(self.config.hierarchy_leaf_capacity);

        if cell_size_changed {
            self.grid = SpatialGrid::new(self.config.cell_size_meters);
            if matches!(self.backend, BackendStorage::Grid) {
                for (entity, position) in &self.entity_positions {
                    self.grid.insert(*entity, *position);
                }
            }
        }
    }

    pub fn active_mode(&self) -> NeighborSearchMode {
        match self.backend {
            BackendStorage::Grid => NeighborSearchMode::UniformCellField,
            BackendStorage::Tree => NeighborSearchMode::HierarchicalEnvelopeField,
        }
    }

    /// Insert entity and return computed grid cell for compatibility with existing callers.
    pub fn insert(&mut self, entity: Entity, position: Vec2) -> (i32, i32) {
        let old = self.entity_positions.insert(entity, position);

        match self.backend {
            BackendStorage::Grid => {
                if let Some(old_pos) = old {
                    let old_cell = self.grid.world_to_grid(old_pos);
                    self.grid.remove_from_cell(entity, old_cell);
                }
                self.grid.insert(entity, position);
            }
            BackendStorage::Tree => {
                self.tree_dirty = true;
            }
        }

        self.grid.world_to_grid(position)
    }

    /// Update entity position and return current grid cell.
    pub fn update(&mut self, entity: Entity, position: Vec2) -> (i32, i32) {
        let old = self.entity_positions.insert(entity, position);

        match self.backend {
            BackendStorage::Grid => match old {
                Some(old_pos) => {
                    let old_cell = self.grid.world_to_grid(old_pos);
                    let new_cell = self.grid.world_to_grid(position);
                    self.grid.move_entity(entity, old_cell, new_cell);
                }
                None => self.grid.insert(entity, position),
            },
            BackendStorage::Tree => {
                self.tree_dirty = true;
            }
        }

        self.grid.world_to_grid(position)
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(old_pos) = self.entity_positions.remove(&entity) {
            match self.backend {
                BackendStorage::Grid => {
                    let old_cell = self.grid.world_to_grid(old_pos);
                    self.grid.remove_from_cell(entity, old_cell);
                }
                BackendStorage::Tree => {
                    self.tree_dirty = true;
                }
            }
        }
    }

    /// Query candidate entities within radius.
    ///
    /// Returned entities are candidates only; exact distance checks remain a caller responsibility.
    pub fn for_each_neighbor_candidate_in_radius(
        &self,
        position: Vec2,
        radius: f32,
        mut emit: impl FnMut(Entity),
    ) {
        debug_assert!(radius >= 0.0, "Negative radius query is undefined");

        match self.backend {
            BackendStorage::Grid => {
                for entity in self.grid.get_entities_in_radius(position, radius) {
                    emit(entity);
                }
            }
            BackendStorage::Tree => self.tree.for_each_in_radius(position, radius, emit),
        }
    }

    /// Query candidate entities within radius into a newly allocated vector.
    ///
    /// Use `for_each_neighbor_candidate_in_radius` in hot paths to avoid per-query allocations.
    pub fn query_radius(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        let mut out = Vec::new();
        self.for_each_neighbor_candidate_in_radius(position, radius, |entity| out.push(entity));
        out
    }

    /// Get the cell size in meters.
    pub fn cell_size(&self) -> f32 {
        self.config.cell_size_meters
    }

    fn rebuild_grid(&mut self) {
        self.grid.clear();
        for (entity, position) in &self.entity_positions {
            self.grid.insert(*entity, *position);
        }
    }

    fn rebuild_tree_if_needed(&mut self) {
        if matches!(self.backend, BackendStorage::Tree) && self.tree_dirty {
            self.tree.rebuild_from_positions(&self.entity_positions);
            self.tree_dirty = false;
        }
    }

    fn switch_backend(&mut self, backend: BackendStorage) {
        self.backend = backend;
        match self.backend {
            BackendStorage::Grid => self.rebuild_grid(),
            BackendStorage::Tree => {
                self.tree_dirty = true;
                self.rebuild_tree_if_needed();
            }
        }
        self.frames_since_switch = 0;
    }

    fn estimate_entities_per_cell(&self) -> f32 {
        let n = self.entity_positions.len();
        if n == 0 {
            return 0.0;
        }

        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for position in self.entity_positions.values() {
            min.x = min.x.min(position.x);
            min.y = min.y.min(position.y);
            max.x = max.x.max(position.x);
            max.y = max.y.max(position.y);
        }

        let dx = (max.x - min.x).max(self.config.cell_size_meters);
        let dy = (max.y - min.y).max(self.config.cell_size_meters);
        let cells_x = (dx / self.config.cell_size_meters).ceil().max(1.0);
        let cells_y = (dy / self.config.cell_size_meters).ceil().max(1.0);
        let estimated_cells = (cells_x * cells_y).max(1.0);
        n as f32 / estimated_cells
    }

    fn preferred_backend_for_auto(&self) -> BackendStorage {
        let n = self.entity_positions.len();
        if n < self.config.adaptive_min_entities_for_hierarchy {
            return BackendStorage::Grid;
        }
        let entities_per_cell = self.estimate_entities_per_cell();
        if entities_per_cell <= self.config.adaptive_sparse_entities_per_cell_threshold {
            BackendStorage::Tree
        } else {
            BackendStorage::Grid
        }
    }

    /// Estimated local packing for adaptive broadphase policy.
    pub fn estimated_entities_per_cell(&self) -> f32 {
        self.estimate_entities_per_cell()
    }

    /// Sync backend mode and rebuild backend storage if needed.
    pub fn prepare_for_queries(&mut self) {
        self.frames_since_switch = self.frames_since_switch.saturating_add(1);

        let target_backend = match self.config.mode {
            NeighborSearchMode::UniformCellField => BackendStorage::Grid,
            NeighborSearchMode::HierarchicalEnvelopeField => BackendStorage::Tree,
            NeighborSearchMode::Adaptive => {
                if self.frames_since_switch < self.config.adaptive_switch_cooldown_frames {
                    self.backend
                } else {
                    self.preferred_backend_for_auto()
                }
            }
        };

        if target_backend != self.backend {
            self.switch_backend(target_backend);
        }

        self.rebuild_tree_if_needed();
    }
}

impl Default for UnifiedSpatialIndex {
    fn default() -> Self {
        Self::from_config(NeighborSearchConfig::default())
    }
}

/// Attach SpatialCell for entities marked SpatiallyIndexed.
pub fn attach_spatial_cells(
    mut commands: Commands,
    mut index: ResMut<UnifiedSpatialIndex>,
    q: Query<(Entity, &Transform), (With<SpatiallyIndexed>, Without<SpatialCell>)>,
) {
    for (e, t) in q.iter() {
        let pos = t.translation.truncate();
        let cell = index.insert(e, pos);
        commands.entity(e).insert(SpatialCell { cell });
    }
}

/// Update index when spatially indexed entities move.
pub fn update_spatial_index(
    mut index: ResMut<UnifiedSpatialIndex>,
    mut q: Query<
        (Entity, &Transform, &mut SpatialCell),
        (With<SpatiallyIndexed>, Changed<Transform>),
    >,
) {
    for (e, t, mut cell) in q.iter_mut() {
        let pos = t.translation.truncate();
        let new_cell = index.update(e, pos);
        cell.cell = new_cell;
    }
}

/// Remove from index when marker removed or entity despawns.
pub fn remove_from_index_on_marker_removed(
    mut index: ResMut<UnifiedSpatialIndex>,
    mut removed: RemovedComponents<SpatiallyIndexed>,
) {
    for e in removed.read() {
        index.remove(e);
    }
}

/// Sync runtime config and prepare active backend for query systems.
pub fn refresh_spatial_index_policy(
    mut index: ResMut<UnifiedSpatialIndex>,
    config: Res<NeighborSearchConfig>,
) {
    index.set_config(&config);
    index.prepare_for_queries();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_mode_radius_query_returns_candidates() {
        let mut world = World::new();
        let mut index = UnifiedSpatialIndex::default();
        index.set_config(&NeighborSearchConfig {
            mode: NeighborSearchMode::UniformCellField,
            ..Default::default()
        });
        index.prepare_for_queries();

        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        index.insert(a, Vec2::new(0.0, 0.0));
        index.insert(b, Vec2::new(500.0, 500.0));
        index.prepare_for_queries();

        let results = index.query_radius(Vec2::new(0.0, 0.0), 25.0);
        assert!(results.contains(&a));
        assert!(!results.contains(&b));
    }

    #[test]
    fn tree_mode_radius_query_returns_candidates() {
        let mut world = World::new();
        let mut index = UnifiedSpatialIndex::default();
        index.set_config(&NeighborSearchConfig {
            mode: NeighborSearchMode::HierarchicalEnvelopeField,
            ..Default::default()
        });
        index.prepare_for_queries();

        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        index.insert(a, Vec2::new(0.0, 0.0));
        index.insert(b, Vec2::new(500.0, 500.0));
        index.prepare_for_queries();

        let results = index.query_radius(Vec2::new(0.0, 0.0), 25.0);
        assert!(results.contains(&a));
        assert!(!results.contains(&b));
    }

    #[test]
    fn adaptive_prefers_hierarchy_for_sparse_large_sets() {
        let mut world = World::new();
        let mut index = UnifiedSpatialIndex::default();
        index.set_config(&NeighborSearchConfig {
            mode: NeighborSearchMode::Adaptive,
            cell_size_meters: 10.0,
            adaptive_min_entities_for_hierarchy: 64,
            adaptive_sparse_entities_per_cell_threshold: 0.20,
            adaptive_switch_cooldown_frames: 1,
            ..Default::default()
        });

        for i in 0..128 {
            let e = world.spawn_empty().id();
            index.insert(e, Vec2::new(i as f32 * 100.0, 0.0));
        }

        index.prepare_for_queries();
        assert_eq!(
            index.active_mode(),
            NeighborSearchMode::HierarchicalEnvelopeField
        );
    }

    #[test]
    fn adaptive_prefers_uniform_cells_for_dense_sets() {
        let mut world = World::new();
        let mut index = UnifiedSpatialIndex::default();
        index.set_config(&NeighborSearchConfig {
            mode: NeighborSearchMode::Adaptive,
            cell_size_meters: 10.0,
            adaptive_min_entities_for_hierarchy: 64,
            adaptive_sparse_entities_per_cell_threshold: 0.20,
            adaptive_switch_cooldown_frames: 1,
            ..Default::default()
        });

        for i in 0..128 {
            let x = (i % 16) as f32;
            let y = (i / 16) as f32;
            let e = world.spawn_empty().id();
            index.insert(e, Vec2::new(x, y));
        }

        index.prepare_for_queries();
        assert_eq!(index.active_mode(), NeighborSearchMode::UniformCellField);
    }
}
