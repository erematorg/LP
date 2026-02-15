use super::mutual::MutualInfo;
use super::shannon::Shannon;

/// Trait for discrete entropy estimators.
pub trait DiscreteEntropyEstimator {
    fn estimate_entropy_bits(&self, values: &[i32]) -> f64;
}

/// Trait for discrete mutual-information estimators.
pub trait DiscreteMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x_values: &[i32], y_values: &[i32]) -> f64;
}

/// Baseline Shannon entropy estimator.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShannonEstimator;

impl DiscreteEntropyEstimator for ShannonEstimator {
    fn estimate_entropy_bits(&self, values: &[i32]) -> f64 {
        Shannon::entropy(values)
    }
}

/// Baseline empirical mutual-information estimator.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmpiricalMutualInformationEstimator;

impl DiscreteMutualInformationEstimator for EmpiricalMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x_values: &[i32], y_values: &[i32]) -> f64 {
        MutualInfo::discrete(x_values, y_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shannon_entropy_for_fair_binary_is_one_bit() {
        let estimator = ShannonEstimator;
        let values = [0, 1, 0, 1];
        let h = estimator.estimate_entropy_bits(&values);
        assert!((h - 1.0).abs() < 1e-9, "expected 1 bit, got {}", h);
    }

    #[test]
    fn shannon_entropy_for_constant_signal_is_zero() {
        let estimator = ShannonEstimator;
        let values = [1, 1, 1, 1];
        let h = estimator.estimate_entropy_bits(&values);
        assert!(h.abs() < 1e-12, "expected near 0, got {}", h);
    }

    #[test]
    fn mutual_information_for_identical_binary_signal_is_one_bit() {
        let estimator = EmpiricalMutualInformationEstimator;
        let x = [0, 1, 0, 1];
        let y = [0, 1, 0, 1];
        let mi = estimator.estimate_mutual_information_bits(&x, &y);
        assert!((mi - 1.0).abs() < 1e-9, "expected 1 bit, got {}", mi);
    }

    #[test]
    fn mutual_information_for_independent_balanced_binary_is_zero() {
        let estimator = EmpiricalMutualInformationEstimator;
        let x = [0, 0, 1, 1];
        let y = [0, 1, 0, 1];
        let mi = estimator.estimate_mutual_information_bits(&x, &y);
        assert!(mi.abs() < 1e-9, "expected near 0, got {}", mi);
    }
}
