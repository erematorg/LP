//! Geometric properties of matter.
//!
//! **Property-based**: Physical dimensions as components.

use bevy::prelude::*;

/// Radius component for spherical or circular matter.
///
/// Units: meters (m)
///
/// **Property-based**: No default radius, must be explicitly set.
/// Used for thermal contact area (A = πr²), softening length, etc.
///
/// **LP-0 assumption**: Assumes spherical particles.
/// Future: Replace with volume/shape descriptors for MPM material points.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Radius {
    /// Radius value in meters.
    pub value: f32,
}
