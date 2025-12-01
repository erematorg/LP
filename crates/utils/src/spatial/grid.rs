use bevy::prelude::*;
use std::collections::HashMap;

/// Sparse spatial hash grid for efficient neighbor queries in an infinite 2D world.
#[derive(Resource, Debug, Clone)]
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<Entity>>,
    pub cell_size: f32,
}

/// Cached grid cell for incremental updates.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
#[reflect(Component)]
pub struct GridCell {
    pub cell: (i32, i32),
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            cells: HashMap::new(),
            cell_size: 50.0,
        }
    }
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }

    pub fn world_to_grid(&self, position: Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        let coords = self.world_to_grid(position);
        let cell = self.cells.entry(coords).or_default();
        if !cell.contains(&entity) {
            cell.push(entity);
        }
    }

    pub fn insert_in_cell(&mut self, entity: Entity, cell: (i32, i32)) {
        let entries = self.cells.entry(cell).or_default();
        if !entries.contains(&entity) {
            entries.push(entity);
        }
    }

    pub fn remove_from_cell(&mut self, entity: Entity, cell: (i32, i32)) {
        if let Some(entries) = self.cells.get_mut(&cell) {
            if let Some(idx) = entries.iter().position(|&e| e == entity) {
                entries.swap_remove(idx);
            }

            if entries.is_empty() {
                self.cells.remove(&cell);
            }
        }
    }

    pub fn move_entity(&mut self, entity: Entity, from: (i32, i32), to: (i32, i32)) {
        if from == to {
            return;
        }

        self.remove_from_cell(entity, from);
        self.insert_in_cell(entity, to);
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Get entities in same cell only
    pub fn get_cell_entities(&self, position: Vec2) -> &[Entity] {
        let coords = self.world_to_grid(position);
        self.cells.get(&coords).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get entities in 3x3 cell area (same cell + 8 neighbors)
    pub fn get_neighbors(&self, position: Vec2) -> Vec<Entity> {
        let (cx, cy) = self.world_to_grid(position);
        let mut neighbors = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell_entities) = self.cells.get(&(cx + dx, cy + dy)) {
                    neighbors.extend_from_slice(cell_entities);
                }
            }
        }

        neighbors
    }

    /// Get entities within radius
    pub fn get_entities_in_radius(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        let min_grid = self.world_to_grid(position - Vec2::splat(radius));
        let max_grid = self.world_to_grid(position + Vec2::splat(radius));
        let mut entities = Vec::new();

        for x in min_grid.0..=max_grid.0 {
            for y in min_grid.1..=max_grid.1 {
                if let Some(cell_entities) = self.cells.get(&(x, y)) {
                    entities.extend_from_slice(cell_entities);
                }
            }
        }

        // Deduplicate in case entities span multiple cells
        entities.sort_unstable_by_key(|e| e.index());
        entities.dedup();
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = SpatialGrid::new(10.0);
        assert_eq!(grid.world_to_grid(Vec2::new(5.0, 5.0)), (0, 0));
        assert_eq!(grid.world_to_grid(Vec2::new(15.0, 25.0)), (1, 2));
        assert_eq!(grid.world_to_grid(Vec2::new(-5.0, -5.0)), (-1, -1));
    }

    #[test]
    fn test_insert_deduplicates() {
        let mut world = World::new();
        let mut grid = SpatialGrid::new(10.0);

        let entity = world.spawn_empty().id();
        grid.insert(entity, Vec2::ZERO);
        grid.insert(entity, Vec2::ZERO);

        let cell_entities = grid.get_cell_entities(Vec2::ZERO);
        assert_eq!(cell_entities.len(), 1);
    }

    #[test]
    fn test_move_entity() {
        let mut world = World::new();
        let mut grid = SpatialGrid::new(10.0);

        let entity = world.spawn_empty().id();
        grid.insert(entity, Vec2::ZERO);
        grid.move_entity(entity, (0, 0), (1, 0));

        assert!(grid.get_cell_entities(Vec2::ZERO).is_empty());
        assert_eq!(grid.get_cell_entities(Vec2::new(10.0, 0.0)).len(), 1);
    }

    #[test]
    fn test_get_neighbors() {
        let mut world = World::new();
        let mut grid = SpatialGrid::new(10.0);

        let mut entities = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                let entity = world.spawn_empty().id();
                grid.insert(entity, Vec2::new(x as f32 * 10.0, y as f32 * 10.0));
                entities.push(entity);
            }
        }

        let neighbors = grid.get_neighbors(Vec2::new(0.0, 0.0));
        assert_eq!(neighbors.len(), 9);
    }

    #[test]
    fn test_get_entities_in_radius() {
        let mut world = World::new();
        let mut grid = SpatialGrid::new(10.0);

        let near = world.spawn_empty().id();
        let far = world.spawn_empty().id();

        grid.insert(near, Vec2::new(0.0, 0.0));
        grid.insert(far, Vec2::new(100.0, 100.0));

        let in_radius = grid.get_entities_in_radius(Vec2::new(0.0, 0.0), 30.0);
        assert!(in_radius.contains(&near));
        assert!(!in_radius.contains(&far));
    }

    #[test]
    fn test_clear() {
        let mut world = World::new();
        let mut grid = SpatialGrid::new(10.0);

        grid.insert(world.spawn_empty().id(), Vec2::new(5.0, 5.0));
        assert_eq!(grid.get_cell_entities(Vec2::new(5.0, 5.0)).len(), 1);

        grid.clear();
        assert_eq!(grid.get_cell_entities(Vec2::new(5.0, 5.0)).len(), 0);
    }
}

