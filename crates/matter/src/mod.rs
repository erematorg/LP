pub mod solids;
pub mod fluids;
pub mod gases;
pub mod plasma;

pub mod prelude {
    // Re-export from state modules
    pub use crate::states::solids::prelude::*;
    pub use crate::states::fluids::prelude::*;
    pub use crate::states::gases::prelude::*;
    pub use crate::states::plasma::prelude::*;
}