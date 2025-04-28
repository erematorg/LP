pub mod fields;
pub mod interactions;

/// The electromagnetism prelude.
///
/// This includes the most common types for electromagnetic systems.
pub mod prelude {
    pub use crate::electromagnetism::fields::{ElectricField, MagneticField};
    pub use crate::electromagnetism::interactions::{
        ElectromagneticWave, MaterialProperties
    };
}