use std::collections::HashMap;

/// Minimal Shannon entropy implementation following energy crate patterns
/// Domain-independent entropy calculation for any discrete system
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
}
