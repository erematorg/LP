use super::mutual::MutualInfo;
use super::shannon::Shannon;

// ── Discrete estimator traits ─────────────────────────────────────────────────

/// Compute entropy from a sequence of discrete integer labels (in bits).
pub trait DiscreteEntropyEstimator {
    fn estimate_entropy_bits(&self, values: &[i32]) -> f64;
}

/// Compute mutual information between two discrete integer sequences (in bits).
pub trait DiscreteMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x_values: &[i32], y_values: &[i32]) -> f64;
}

// ── Continuous estimator traits ───────────────────────────────────────────────
//
// Continuous data is represented as &[Vec<f32>] (N points × D dimensions),
// matching the kNN-based API in shannon.rs and mutual/calculation.rs.
//
// LP / multiplayer usage: plug in the right estimator per signal type without
// changing call sites. Discrete handles genome symbols and material-type counts;
// continuous handles position density fields, velocity distributions, and any
// other physical observable sampled from the MPM grid.

/// Compute differential entropy from a continuous multivariate sample (in bits).
pub trait ContinuousEntropyEstimator {
    fn estimate_entropy_bits(&self, data: &[Vec<f32>]) -> f64;
}

/// Compute mutual information between two continuous multivariate samples (in bits).
pub trait ContinuousMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x: &[Vec<f32>], y: &[Vec<f32>]) -> f64;
}

// ── Discrete implementations ──────────────────────────────────────────────────

/// Baseline maximum-likelihood Shannon entropy estimator for discrete data.
/// Biased downward for small samples; prefer Miller-Madow correction for n < 100.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShannonEstimator;

impl DiscreteEntropyEstimator for ShannonEstimator {
    fn estimate_entropy_bits(&self, values: &[i32]) -> f64 {
        Shannon::entropy(values)
    }
}

/// Baseline empirical mutual-information estimator for discrete data.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmpiricalMutualInformationEstimator;

impl DiscreteMutualInformationEstimator for EmpiricalMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x_values: &[i32], y_values: &[i32]) -> f64 {
        MutualInfo::discrete(x_values, y_values)
    }
}

// ── Continuous implementations (k-NN based) ───────────────────────────────────

/// Kraskov et al. 2004 k-NN entropy estimator for continuous multivariate data.
/// `k` controls the neighbourhood size; k=3..5 works well for most LP use cases.
#[derive(Debug, Clone, Copy)]
pub struct KnnEntropyEstimator {
    pub k: usize,
}

impl Default for KnnEntropyEstimator {
    fn default() -> Self {
        Self { k: 3 }
    }
}

impl ContinuousEntropyEstimator for KnnEntropyEstimator {
    fn estimate_entropy_bits(&self, data: &[Vec<f32>]) -> f64 {
        Shannon::continuous_entropy(data, self.k)
    }
}

/// Kraskov et al. 2004 k-NN mutual information estimator for continuous data.
/// `k` controls the neighbourhood size; k=3..5 works well for most LP use cases.
#[derive(Debug, Clone, Copy)]
pub struct KnnMutualInformationEstimator {
    pub k: usize,
}

impl Default for KnnMutualInformationEstimator {
    fn default() -> Self {
        Self { k: 3 }
    }
}

impl ContinuousMutualInformationEstimator for KnnMutualInformationEstimator {
    fn estimate_mutual_information_bits(&self, x: &[Vec<f32>], y: &[Vec<f32>]) -> f64 {
        MutualInfo::continuous_knn(x, y, self.k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── discrete ─────────────────────────────────────────────────────────────

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

    // ── continuous ────────────────────────────────────────────────────────────

    #[test]
    fn knn_entropy_estimator_positive_for_spread_data() {
        let estimator = KnnEntropyEstimator::default();
        // 20 points spread across 1D — entropy should be clearly positive
        let data: Vec<Vec<f32>> = (0..20).map(|i| vec![i as f32 * 0.5]).collect();
        let h = estimator.estimate_entropy_bits(&data);
        assert!(
            h > 0.0,
            "expected positive entropy for spread data, got {}",
            h
        );
    }

    #[test]
    fn knn_mi_estimator_nonzero_for_correlated_data() {
        let estimator = KnnMutualInformationEstimator::default();
        // x and y perfectly correlated → MI should be positive
        let x: Vec<Vec<f32>> = (0..20).map(|i| vec![i as f32]).collect();
        let y: Vec<Vec<f32>> = (0..20).map(|i| vec![i as f32 * 2.0]).collect();
        let mi = estimator.estimate_mutual_information_bits(&x, &y);
        assert!(
            mi > 0.0,
            "expected positive MI for correlated data, got {}",
            mi
        );
    }
}
