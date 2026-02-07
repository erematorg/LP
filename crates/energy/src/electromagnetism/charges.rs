//! Coulomb forces between point charges.
//!
//! **LP-0 SCAFFOLDING**: Particle-particle Coulomb interactions.
//! Future: Will be replaced by grid-based Poisson solve (ρ → φ → E).
//!
//! Physics: F = k·q₁·q₂/r² (Coulomb's law)
//! Complexity: O(N) with SpatialGrid
//! Conservation: Force-only (EM potential energy = 0 for LP-0)

use bevy::prelude::*;
use forces::core::newton_laws::AppliedForce;
use std::collections::HashMap;
use utils::{SpatiallyIndexed, UnifiedSpatialIndex, force_switch};

/// Electric charge component.
///
/// Units: Coulombs (C)
///
/// **LP-0 assumption**: Point charges for validation.
/// Future: Replace with charge density fields on grid.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Charge {
    /// Charge value in Coulombs.
    ///
    /// Positive = proton-like, Negative = electron-like.
    pub value: f32,
}

impl Charge {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn positive(value: f32) -> Self {
        Self { value: value.abs() }
    }

    pub fn negative(value: f32) -> Self {
        Self {
            value: -value.abs(),
        }
    }
}

/// Softening length for singularity avoidance.
///
/// **Property-based**: No hardcoded epsilon values.
/// When r < softening_length, force is smoothly reduced to avoid 1/r² singularity.
///
/// Units: meters (m)
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct SofteningLength {
    /// Minimum distance for force calculation.
    ///
    /// Typically ~0.01m for particle-scale simulations.
    pub value: f32,
}

impl Default for SofteningLength {
    fn default() -> Self {
        Self { value: 0.01 }
    }
}

/// Configuration for Coulomb force system.
#[derive(Resource, Debug, Clone)]
pub struct CoulombConfig {
    /// Coulomb constant k = 1/(4πε₀).
    ///
    /// **UNITS**: Newtons × meters² / Coulombs² (N·m²/C²)
    /// **Default**: 8.99×10⁹ N·m²/C² (vacuum permittivity ε₀ = 8.854×10⁻¹² F/m)
    /// **IRL PHYSICS**: k = 8.987551792×10⁹ N·m²/C² (exact)
    pub coulomb_constant: f32,

    /// Cutoff radius for Coulomb interactions.
    ///
    /// **UNITS**: meters (m)
    /// **PERFORMANCE APPROXIMATION**: IRL Coulomb has infinite range in vacuum.
    /// Cutoff is computational approximation, not physics.
    /// **Default**: 20m (tunable based on system density and interaction scales)
    pub cutoff_radius: f32,

    /// Start of force-switch transition.
    ///
    /// **UNITS**: meters (m)
    /// **Default**: 0.8 × cutoff_radius (C¹ smooth cutoff for numerical stability)
    pub switch_on_radius: f32,
}

impl Default for CoulombConfig {
    fn default() -> Self {
        let cutoff = 20.0;
        Self {
            coulomb_constant: 8.99e9, // N⋅m²/C²
            cutoff_radius: cutoff,
            switch_on_radius: 0.8 * cutoff,
        }
    }
}

/// Mark charged entities for spatial indexing.
///
/// **Phase A2**: Inject SpatiallyIndexed marker for UnifiedSpatialIndex.
/// This allows utils crate to manage spatial indexing without depending on Charge.
pub fn mark_charged_entities_spatially_indexed(
    mut commands: Commands,
    q: Query<Entity, (With<Charge>, Without<SpatiallyIndexed>)>,
) {
    for e in q.iter() {
        commands.entity(e).insert(SpatiallyIndexed);
    }
}

/// Apply Coulomb electrostatic forces between charged particles.
///
/// **LP-0 SCAFFOLDING**: Pairwise particle-particle Coulomb interactions (O(N) via spatial hash grid).
/// **TEMPORARY**: Will be replaced by grid-based Poisson solve (ρ → φ → E) in LP-1.
///
/// **PHYSICS**: Coulomb's Law F = k·q₁·q₂/r² (Newtons)
/// - F: Force magnitude (N)
/// - k: Coulomb constant = 8.99×10⁹ N·m²/C² (vacuum permittivity)
/// - q₁, q₂: Charges (Coulombs)
/// - r: Distance (meters)
/// - Direction: Along line between charges (repulsive if same sign, attractive if opposite)
///
/// **APPROXIMATIONS**:
/// - Cutoff radius: 20m default (performance hack, IRL Coulomb has infinite range in vacuum)
/// - Softening: 0.01m default (singularity avoidance for r→0)
/// - Potential energy: Not tracked (force-only, PE = 0 in LP-0)
/// - Pair-once guarantee: Only processes pairs where entity_b.index() > entity_a.index() to avoid double-counting
///
/// **CONSERVATION**: Momentum conserved (F_ab = -F_ba, Newton's 3rd law).
/// Energy NOT conserved (PE missing from accounting).
pub fn apply_coulomb_pairwise_forces(
    mut charges: Query<(
        Entity,
        &Charge,
        &Transform,
        Option<&SofteningLength>,
        &mut AppliedForce,
    )>,
    index: Res<UnifiedSpatialIndex>,
    config: Res<CoulombConfig>,
) {
    // **LP-0 SCAFFOLDING**: Pairwise particle-particle Coulomb forces.
    // Future: Grid-based Poisson solve (ρ → φ → E).

    // Stage charges into map to avoid nested query
    let mut charge_data: HashMap<Entity, (f32, Vec2, f32)> = HashMap::new();
    for (entity, charge, trans, softening, _) in charges.iter() {
        let pos = trans.translation.truncate();

        // No silent defaults: require SofteningLength
        let Some(soft) = softening else {
            #[cfg(debug_assertions)]
            panic!(
                "Entity {:?} missing SofteningLength for Coulomb forces",
                entity
            );

            #[cfg(not(debug_assertions))]
            {
                static LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    warn!("Skipping charged entities missing SofteningLength (logged once)");
                }
                continue;
            }
        };

        charge_data.insert(entity, (charge.value, pos, soft.value));
    }

    // Iterate pairs via UnifiedSpatialIndex
    for (entity_a, (charge_a, pos_a, soft_a)) in charge_data.iter() {
        // Find neighbors within cutoff using UnifiedSpatialIndex (O(N) average)
        for entity_b in index.query_radius(*pos_a, config.cutoff_radius) {
            // **Pair-once guarantee**: Only process pairs where B > A
            if entity_b.index() <= entity_a.index() {
                continue;
            }

            // Get data for entity B from staged map
            let Some((charge_b, pos_b, soft_b)) = charge_data.get(&entity_b) else {
                continue;
            };

            let r_vec = *pos_b - *pos_a;
            let r = r_vec.length();

            // Property-based softening: use max of the two particles' softening lengths
            let softening = soft_a.max(*soft_b);
            if r < softening || r >= config.cutoff_radius {
                continue;
            }

            // Coulomb force: F_on_A = -k·q₁·q₂/r² · r̂_AB
            // Same-sign charges (k_qq > 0) → repulsive (force pushes A away from B)
            // Opposite-sign charges (k_qq < 0) → attractive (force pulls A toward B)
            let k_qq = config.coulomb_constant * charge_a * charge_b;
            let force_magnitude = k_qq / r.powi(2);
            let force_bare = -(force_magnitude / r) * r_vec;

            // Apply C¹ force-switch for smooth cutoff
            let switch = force_switch(r, config.switch_on_radius, config.cutoff_radius);
            let force_2d = force_bare * switch;
            let force = force_2d.extend(0.0); // Convert to Vec3 for AppliedForce

            // Apply forces symmetrically (Newton's 3rd law)
            if let Ok((_, _, _, _, mut force_a)) = charges.get_mut(*entity_a) {
                force_a.force += force;
            }
            if let Ok((_, _, _, _, mut force_b)) = charges.get_mut(entity_b) {
                force_b.force -= force; // F_ba = -F_ab
            }

            // **LP-0**: EM potential energy = 0 (force-only).
            // Future: Track U(r) = integral of switched force for energy conservation.
        }
    }
}
