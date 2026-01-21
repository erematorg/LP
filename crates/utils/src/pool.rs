use bevy::prelude::*;

/// Marker component for entities currently parked in an EntityPool.
///
/// **Semantics:**
/// - Entities in the pool have `Pooled` and no other gameplay components
/// - `acquire()` removes `Pooled`, making the entity active
/// - `release()` inserts `Pooled` and strips other components, parking it for reuse
///
/// **Usage:** Use `Without<Pooled>` in queries to see only active entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
#[reflect(Component)]
pub struct Pooled;

/// Resource for managing reusable entity pools
///
/// Prevents allocation/deallocation overhead by recycling entities.
/// Common use cases: particles, bullets, VFX, temporary calculations.
///
/// ## Usage
/// ```ignore
/// // Setup
/// commands.insert_resource(EntityPool::new(100));
///
/// // Acquire active entity from pool (no Pooled marker)
/// let entity = pool.acquire(&mut commands);
/// commands.entity(entity).insert(MyComponent);
///
/// // Return entity to pool (adds Pooled marker, strips other components)
/// pool.release(&mut commands, entity);
/// ```
///
/// ## Query Pattern
/// ```ignore
/// // See only active entities (excludes pooled ones)
/// fn system(query: Query<&MyComponent, Without<Pooled>>) { }
/// ```
///
/// ## Performance
/// - Prewarm pool to avoid runtime allocations
/// - Zero external dependencies
#[derive(Resource, Debug)]
pub struct EntityPool {
    available: Vec<Entity>,
    capacity: usize,
}

impl EntityPool {
    /// Create new pool with initial capacity hint
    pub fn new(capacity: usize) -> Self {
        Self {
            available: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Get entity from pool (spawns new if empty)
    ///
    /// Returns an active entity without the `Pooled` marker.
    /// Components can be added to this entity immediately.
    pub fn acquire(&mut self, commands: &mut Commands) -> Entity {
        if let Some(entity) = self.available.pop() {
            // Entity leaves pool: remove Pooled marker to make it active
            commands.entity(entity).remove::<Pooled>();
            entity
        } else {
            // Fresh active entity (not pooled yet)
            commands.spawn_empty().id()
        }
    }

    /// Return entity to pool for reuse
    ///
    /// **IMPORTANT:** Strips ALL components except Pooled (including Transform, children, etc.)
    /// Reused entities start with a clean slate - reinitialize all components on acquire.
    ///
    /// Entity remains valid but inactive.
    ///
    /// **Performance Note:** For high-frequency spawn/despawn (1000+ per frame), archetype
    /// migration overhead may become significant. Consider soft-pooling (Inactive marker) instead.
    pub fn release(&mut self, commands: &mut Commands, entity: Entity) {
        // Ensure Pooled marker is present, then strip everything else
        commands.entity(entity).insert(Pooled).retain::<Pooled>();

        if self.available.len() < self.capacity {
            self.available.push(entity);
        } else {
            // Pool full, despawn excess
            commands.entity(entity).despawn();
        }
    }

    /// Pre-spawn N entities to avoid runtime allocation spikes
    ///
    /// Respects capacity limit - will not exceed pool capacity.
    pub fn prewarm(&mut self, commands: &mut Commands, count: usize) {
        let actual_count = count.min(self.capacity.saturating_sub(self.available.len()));
        for _ in 0..actual_count {
            let entity = commands.spawn(Pooled).id();
            self.available.push(entity);
        }
    }

    /// Number of entities ready for reuse
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Clear all pooled entities (despawn them)
    pub fn clear(&mut self, commands: &mut Commands) {
        for entity in self.available.drain(..) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct TestComponent(u32);

    #[test]
    fn test_acquire_from_empty_pool() {
        let mut world = World::new();
        let mut pool = EntityPool::new(10);

        let entity = pool.acquire(&mut world.commands());
        world.flush();

        // Acquired entities are active (no Pooled marker)
        assert!(world.get::<Pooled>(entity).is_none());
    }

    #[test]
    fn test_release_and_reacquire() {
        let mut world = World::new();
        let mut pool = EntityPool::new(10);

        let entity1 = pool.acquire(&mut world.commands());
        world.flush();

        pool.release(&mut world.commands(), entity1);
        world.flush();

        let entity2 = pool.acquire(&mut world.commands());

        // Should reuse same entity
        assert_eq!(entity1, entity2);
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_prewarm() {
        let mut world = World::new();
        let mut pool = EntityPool::new(5);

        pool.prewarm(&mut world.commands(), 5);
        world.flush();

        assert_eq!(pool.available_count(), 5);

        // Acquire prewarmed entities
        for _ in 0..5 {
            pool.acquire(&mut world.commands());
        }

        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_release_strips_components() {
        let mut world = World::new();
        let mut pool = EntityPool::new(10);

        let entity = pool.acquire(&mut world.commands());
        world.flush();

        world.entity_mut(entity).insert(TestComponent(42));

        pool.release(&mut world.commands(), entity);
        world.flush();

        // Component should be removed
        assert!(world.get::<TestComponent>(entity).is_none());
        assert!(world.get::<Pooled>(entity).is_some());
    }

    #[test]
    fn test_pool_capacity_limit() {
        let mut world = World::new();
        let mut pool = EntityPool::new(2);

        let e1 = pool.acquire(&mut world.commands());
        let e2 = pool.acquire(&mut world.commands());
        let e3 = pool.acquire(&mut world.commands());
        world.flush();

        pool.release(&mut world.commands(), e1);
        pool.release(&mut world.commands(), e2);
        pool.release(&mut world.commands(), e3);
        world.flush();

        // Only 2 should be pooled (capacity), third despawned
        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn test_clear_pool() {
        let mut world = World::new();
        let mut pool = EntityPool::new(10);

        pool.prewarm(&mut world.commands(), 5);
        world.flush();

        pool.clear(&mut world.commands());
        world.flush();

        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_prewarm_respects_capacity() {
        let mut world = World::new();
        let mut pool = EntityPool::new(5);

        // Try to prewarm more than capacity
        pool.prewarm(&mut world.commands(), 10);
        world.flush();

        // Should only prewarm up to capacity
        assert_eq!(pool.available_count(), 5);

        // Prewarm again should not add more
        pool.prewarm(&mut world.commands(), 5);
        world.flush();

        assert_eq!(pool.available_count(), 5);
    }
}
