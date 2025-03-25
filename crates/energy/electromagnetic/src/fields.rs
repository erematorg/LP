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
    
    /// Calculate the electric field from a point charge
    pub fn from_point_charge(charge: f32, charge_position: Vec3, field_position: Vec3) -> Self {
        // Coulomb's constant (k = 1/4πε₀) in simplified units
        const K: f32 = 8.99e9;
        
        let r = field_position - charge_position;
        let distance_squared = r.length_squared();
        
        if distance_squared < 1e-10 {
            return Self::new(Vec3::ZERO, field_position);
        }
        
        let direction = r.normalize();
        let magnitude = K * charge / distance_squared;
        
        Self::new(direction * magnitude, field_position)
    }
    
    /// Combine two electric fields (superposition principle)
    pub fn superpose(&self, other: &ElectricField) -> Self {
        // Fields can only be superposed at the same position
        assert!((self.position - other.position).length() < 1e-6, 
                "Cannot superpose fields at different positions");
        
        Self::new(self.field + other.field, self.position)
    }
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
    
    /// Calculate the magnetic field from a current element
    pub fn from_current_element(current: f32, current_direction: Vec3, 
                              current_position: Vec3, field_position: Vec3) -> Self {
        // Magnetic constant (μ₀/4π) in simplified units
        const MU_0_DIV_4PI: f32 = 1e-7;
        
        let r = field_position - current_position;
        let distance = r.length();
        
        if distance < 1e-10 {
            return Self::new(Vec3::ZERO, field_position);
        }
        
        let r_unit = r / distance;
        
        // Biot-Savart law: dB = (μ₀/4π) * (I dl × r̂) / r²
        let field = current_direction.cross(r_unit) * (MU_0_DIV_4PI * current / (distance * distance));
        
        Self::new(field, field_position)
    }
    
    /// Combine two magnetic fields (superposition principle)
    pub fn superpose(&self, other: &MagneticField) -> Self {
        // Fields can only be superposed at the same position
        assert!((self.position - other.position).length() < 1e-6, 
                "Cannot superpose fields at different positions");
        
        Self::new(self.field + other.field, self.position)
    }
}