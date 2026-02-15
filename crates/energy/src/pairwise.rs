use bevy::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use utils::UnifiedSpatialIndex;

/// Controls determinism level for pairwise neighbor emission.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct PairwiseDeterminismConfig {
    /// When true, neighbor candidates are collected and sorted by entity id before processing.
    /// This enables exact replay determinism at additional per-frame cost.
    pub strict_neighbor_order: bool,
}

impl Default for PairwiseDeterminismConfig {
    fn default() -> Self {
        Self {
            strict_neighbor_order: false,
        }
    }
}

/// Prepare a staging map for a new frame without reallocating from scratch.
pub(crate) fn prepare_staging_map<K: Eq + Hash, V>(
    map: &mut HashMap<K, V>,
    estimated_items: usize,
) {
    map.clear();
    map.reserve(estimated_items);
}

/// Build deterministic entity iteration order from keys.
pub(crate) fn prepare_sorted_entities_from_keys(
    sorted_entities: &mut Vec<Entity>,
    keys: impl IntoIterator<Item = Entity>,
) {
    sorted_entities.clear();
    sorted_entities.extend(keys);
    sorted_entities.sort_by_key(|e| e.to_bits());
}

/// Pair-once check: returns true only if `b` should be processed when iterating from `a`.
pub(crate) fn is_forward_entity_pair(a: Entity, b: Entity) -> bool {
    b.to_bits() > a.to_bits()
}

/// Emit neighbor candidates in either fast path order or strict deterministic order.
pub(crate) fn for_each_neighbor_candidate(
    index: &UnifiedSpatialIndex,
    position: Vec2,
    radius: f32,
    strict_neighbor_order: bool,
    scratch: &mut Vec<Entity>,
    mut emit: impl FnMut(Entity),
) {
    if strict_neighbor_order {
        scratch.clear();
        index.for_each_neighbor_candidate_in_radius(position, radius, |entity| {
            scratch.push(entity);
        });
        scratch.sort_by_key(|e| e.to_bits());
        for &entity in scratch.iter() {
            emit(entity);
        }
    } else {
        index.for_each_neighbor_candidate_in_radius(position, radius, emit);
    }
}
