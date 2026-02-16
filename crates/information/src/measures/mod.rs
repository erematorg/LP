pub mod divergence;
pub mod estimators;
mod knn_estimators;
pub mod mutual;
pub mod shannon; // Internal k-NN core for continuous estimators

// Re-export main plugin
pub use mutual::MutualInformationPlugin;

pub mod prelude {
    pub use super::divergence::KLDivergence;
    pub use super::estimators::{
        DiscreteEntropyEstimator, DiscreteMutualInformationEstimator,
        EmpiricalMutualInformationEstimator, ShannonEstimator,
    };
    pub use super::mutual::*;
    pub use super::shannon::Shannon;
}
