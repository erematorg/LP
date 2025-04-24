pub mod fields;
pub mod interactions;

pub mod prelude {
    pub use crate::fields::{ElectricField, MagneticField};
    pub use crate::interactions::{ElectromagneticWave, MaterialProperties};
}