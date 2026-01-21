//! Physical units and coordinate system conversions.
//!
//! **LP Units Convention**:
//! - Physics simulation: SI units (meters, kilograms, seconds, Kelvin, Coulombs)
//! - Rendering/world: Bevy unit-less space
//! - Conversion: PhysicsScale resource defines mapping

use bevy::prelude::*;

/// Physical scale mapping between rendering and physics coordinates.
///
/// **LP-0**: All physics uses SI units internally.
/// **Rendering**: Bevy uses unit-less coordinates (typically pixels or world units).
///
/// **Example**: If 10 Bevy units = 1 meter, set `render_units_per_meter = 10.0`
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct PhysicsScale {
    /// Rendering units per meter (e.g., 10.0 means 10 pixels = 1 meter).
    ///
    /// **UNITS**: dimensionless (render units / meter)
    /// **Default**: 1.0 (assumes 1:1 mapping - usually incorrect, will warn in debug)
    pub render_units_per_meter: f32,
}

impl Default for PhysicsScale {
    fn default() -> Self {
        Self {
            render_units_per_meter: 1.0, // Default 1:1, user should configure
        }
    }
}

/// Convert rendering position to physics position (meters).
///
/// **Usage**:
/// ```ignore
/// let physics_pos = render_to_physics(sprite_pos, &scale);
/// ```
#[inline]
pub fn render_to_physics(render_pos: Vec2, scale: &PhysicsScale) -> Vec2 {
    render_pos / scale.render_units_per_meter
}

/// Convert physics position (meters) to rendering position.
///
/// **Usage**:
/// ```ignore
/// let sprite_pos = physics_to_render(physics_pos, &scale);
/// ```
#[inline]
pub fn physics_to_render(physics_pos: Vec2, scale: &PhysicsScale) -> Vec2 {
    physics_pos * scale.render_units_per_meter
}

/// Warn if PhysicsScale not configured (debug builds only).
///
/// **Purpose**: Catch 1:1 default scale assumption (usually wrong).
#[cfg(debug_assertions)]
pub fn validate_physics_scale(scale: Res<PhysicsScale>) {
    if (scale.render_units_per_meter - 1.0).abs() < f32::EPSILON {
        warn!(
            "PhysicsScale not configured - using 1:1 render/physics mapping.\n\
             Set PhysicsScale.render_units_per_meter for correct scaling.\n\
             Example: app.insert_resource(PhysicsScale {{ render_units_per_meter: 10.0 }});"
        );
    }
}
