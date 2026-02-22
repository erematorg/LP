use super::knn_estimators;
use std::collections::HashMap;

/// Minimal Shannon entropy implementation following energy crate patterns
/// Domain-independent entropy calculation for any discrete system
/// Extended with continuous k-NN estimators (Kraskov et al. 2004)
pub struct Shannon;

impl Shannon {
    /// Calculate Shannon entropy H(X) = -Σ p(x) log₂ p(x)
    /// Returns entropy in bits - domain agnostic
    pub fn entropy(values: &[i32]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let n = values.len() as f64;
        let mut counts = HashMap::new();

        for &value in values {
            *counts.entry(value).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in counts.values() {
            let p = *count as f64 / n;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Calculate entropy directly from probability distribution
    pub fn entropy_from_probs(probabilities: &[f64]) -> f64 {
        let mut entropy = 0.0;
        for &p in probabilities {
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Calculate conditional entropy H(X|Y) = H(X,Y) - H(Y)
    /// Measures uncertainty in X given knowledge of Y
    pub fn conditional_entropy(x_values: &[i32], y_values: &[i32]) -> f64 {
        assert_eq!(
            x_values.len(),
            y_values.len(),
            "X and Y must have same length"
        );

        if x_values.is_empty() {
            return 0.0;
        }

        let joint_entropy = Self::joint_entropy(x_values, y_values);
        let y_entropy = Self::entropy(y_values);

        joint_entropy - y_entropy
    }

    /// Calculate joint entropy H(X,Y) = -Σ p(x,y) log₂ p(x,y)
    pub fn joint_entropy(x_values: &[i32], y_values: &[i32]) -> f64 {
        assert_eq!(
            x_values.len(),
            y_values.len(),
            "X and Y must have same length"
        );

        if x_values.is_empty() {
            return 0.0;
        }

        let n = x_values.len() as f64;
        let mut joint_counts = std::collections::HashMap::new();

        for (&x, &y) in x_values.iter().zip(y_values) {
            *joint_counts.entry((x, y)).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in joint_counts.values() {
            let p = *count as f64 / n;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    // ========== CONTINUOUS ENTROPY (K-NN ESTIMATORS) ==========
    // Following Kraskov, Stögbauer, Grassberger 2004
    // Port of NPEET: https://github.com/gregversteeg/NPEET

    /// Continuous entropy estimation via k-NN (Kraskov et al.)
    /// Input: data points as vectors of f32 (e.g., [[1.3], [3.7], [5.1]])
    /// k: number of nearest neighbors (default 3)
    /// Returns entropy in nats (natural log), divide by ln(2) for bits
    pub fn continuous_entropy(data: &[Vec<f32>], k: usize) -> f64 {
        if data.is_empty() || data[0].is_empty() {
            return 0.0;
        }

        assert!(k < data.len(), "k must be smaller than number of points");

        let n = data.len() as f64;
        let d = data[0].len() as f64;

        // Add noise to avoid exact duplicates
        let noisy_data = knn_estimators::add_noise(data, 1e-5);

        // Build distance matrix
        let distances = knn_estimators::build_distance_matrix(&noisy_data);

        // Get k-NN distances
        let knn_dists = knn_estimators::knn_distances(&distances, k);

        // Kraskov formula: H(X) = (digamma(n) - digamma(k) + d*<ln(r_k)>) / ln(base)
        let const_term = knn_estimators::digamma(n) - knn_estimators::digamma(k as f64)
            + d * std::f64::consts::LN_2;

        let avg_log_dist: f64 = knn_dists
            .iter()
            .filter(|&&d| d > 1e-9)
            .map(|&d| (d as f64).ln())
            .sum::<f64>()
            / knn_dists.len() as f64;

        (const_term + d * avg_log_dist) / std::f64::consts::LN_2 // Convert to bits
    }

    /// Conditional entropy H(X|Y) = H(X,Y) - H(Y) for continuous data
    pub fn continuous_conditional_entropy(x: &[Vec<f32>], y: &[Vec<f32>], k: usize) -> f64 {
        assert_eq!(x.len(), y.len(), "X and Y must have same length");

        let xy_entropy = Self::continuous_joint_entropy(x, y, k);
        let y_entropy = Self::continuous_entropy(y, k);

        xy_entropy - y_entropy
    }

    /// Joint entropy H(X,Y) for continuous data
    pub fn continuous_joint_entropy(x: &[Vec<f32>], y: &[Vec<f32>], k: usize) -> f64 {
        assert_eq!(x.len(), y.len(), "X and Y must have same length");

        // Stack X and Y horizontally
        let mut xy = vec![];
        for (xi, yi) in x.iter().zip(y.iter()) {
            let mut combined = xi.clone();
            combined.extend(yi.iter());
            xy.push(combined);
        }

        Self::continuous_entropy(&xy, k)
    }
}
