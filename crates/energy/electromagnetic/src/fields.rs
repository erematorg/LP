use bevy::prelude::*;

/// Represents an electric field component
#[derive(Debug, Clone, Copy, Component)]
pub struct ElectricField {
    pub field: Vec3,
    pub position: Vec3,
}

impl ElectricField {
    pub fn new(field: Vec3, position: Vec3) -> Self { Self { field, position } }
    pub fn strength(&self) -> f32 { self.field.length() }
}

/// Represents a magnetic field component
#[derive(Debug, Clone, Copy, Component)]
pub struct MagneticField {
    pub field: Vec3,
    pub position: Vec3,
}

impl MagneticField {
    pub fn new(field: Vec3, position: Vec3) -> Self { Self { field, position } }
    pub fn strength(&self) -> f32 { self.field.length() }
}