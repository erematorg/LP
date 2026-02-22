use super::super::knn_estimators;
use std::collections::HashMap;

/// Core mutual information calculation for discrete and continuous variables
///
/// Mutual Information I(X;Y) measures how much knowing X tells us about Y
/// Formula: I(X;Y) = H(X) + H(Y) - H(X,Y)
/// Extended with continuous k-NN estimators (Kraskov et al. 2004)
pub struct MutualInfo;

impl MutualInfo {
    /// Calculate mutual information for discrete variables
    /// Returns I(X;Y) in bits
    pub fn discrete(x_values: &[i32], y_values: &[i32]) -> f64 {
        assert_eq!(
            x_values.len(),
            y_values.len(),
            "X and Y must have same length"
        );

        let n = x_values.len() as f64;
        if n == 0.0 {
            return 0.0;
        }

        // Build frequency tables
        let mut joint_counts = HashMap::new();
        let mut x_counts = HashMap::new();
        let mut y_counts = HashMap::new();

        for (&x, &y) in x_values.iter().zip(y_values) {
            *joint_counts.entry((x, y)).or_insert(0) += 1;
            *x_counts.entry(x).or_insert(0) += 1;
            *y_counts.entry(y).or_insert(0) += 1;
        }

        // Calculate MI using probability ratios
        let mut mi = 0.0;
        for ((x, y), &joint_count) in &joint_counts {
            let p_xy = joint_count as f64 / n;
            let p_x = *x_counts.get(x).unwrap() as f64 / n;
            let p_y = *y_counts.get(y).unwrap() as f64 / n;

            // Only add to MI if all probabilities are positive
            if p_xy > 0.0 {
                mi += p_xy * (p_xy / (p_x * p_y)).log2();
            }
        }

        mi.max(0.0) // Ensure non-negative due to floating point errors
    }

    /// Calculate mutual information for continuous variables using binning
    /// bins: number of bins for histogram discretization
    pub fn continuous(x_values: &[f64], y_values: &[f64], bins: usize) -> f64 {
        assert_eq!(
            x_values.len(),
            y_values.len(),
            "X and Y must have same length"
        );

        if x_values.is_empty() || bins == 0 {
            return 0.0;
        }

        // Find ranges for binning
        let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Handle edge case where all values are the same
        if x_min == x_max || y_min == y_max {
            return 0.0;
        }

        let x_step = (x_max - x_min) / bins as f64;
        let y_step = (y_max - y_min) / bins as f64;

        // Convert to discrete bins
        let x_bins: Vec<i32> = x_values
            .iter()
            .map(|&x| ((x - x_min) / x_step).floor().min(bins as f64 - 1.0) as i32)
            .collect();
        let y_bins: Vec<i32> = y_values
            .iter()
            .map(|&y| ((y - y_min) / y_step).floor().min(bins as f64 - 1.0) as i32)
            .collect();

        // Use discrete calculation on binned data
        Self::discrete(&x_bins, &y_bins)
    }

    /// Estimate mutual information with bias correction
    /// Uses Miller-Madow bias correction for small samples
    pub fn corrected_discrete(x_values: &[i32], y_values: &[i32]) -> f64 {
        let raw_mi = Self::discrete(x_values, y_values);

        let n = x_values.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }

        // Count unique joint states for Miller-Madow correction
        let joint_unique = x_values
            .iter()
            .zip(y_values)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;

        // Miller-Madow bias correction: (K-1)/(2N) where K is unique states
        let bias_correction = (joint_unique - 1.0) / (2.0 * n);
        (raw_mi - bias_correction).max(0.0)
    }

    // ========== CONTINUOUS MI (K-NN ESTIMATORS) ==========
    // Following Kraskov, Stögbauer, Grassberger 2004

    /// Continuous mutual information via k-NN (Kraskov et al.)
    /// Input: data as vectors of f32
    /// I(X;Y) = digamma(k) - <digamma(nx)> - <digamma(ny)> + digamma(n)
    /// where nx, ny are counts in hypercubes around each joint point
    /// k: number of nearest neighbors (default 3)
    /// Returns MI in bits
    pub fn continuous_knn(x: &[Vec<f32>], y: &[Vec<f32>], k: usize) -> f64 {
        assert_eq!(x.len(), y.len(), "X and Y must have same length");
        if x.is_empty() || x[0].is_empty() || y[0].is_empty() {
            return 0.0;
        }

        let n = x.len() as f64;
        assert!(k < x.len(), "k must be smaller than number of points");

        // Stack X and Y horizontally for joint space
        let mut xy = vec![];
        for (xi, yi) in x.iter().zip(y.iter()) {
            let mut combined = xi.clone();
            combined.extend(yi.iter());
            xy.push(combined);
        }

        // Add noise to avoid exact duplicates
        let x_noisy = knn_estimators::add_noise(x, 1e-5);
        let y_noisy = knn_estimators::add_noise(y, 1e-5);
        let xy_noisy = knn_estimators::add_noise(&xy, 1e-5);

        // Build distance matrices
        let x_dists = knn_estimators::build_distance_matrix(&x_noisy);
        let y_dists = knn_estimators::build_distance_matrix(&y_noisy);
        let xy_dists = knn_estimators::build_distance_matrix(&xy_noisy);

        // Get k-NN distances in joint space
        let joint_knn = knn_estimators::knn_distances(&xy_dists, k);

        // For each point, count neighbors within joint distance in X and Y spaces
        let mut avgdigamma_x = 0.0;
        let mut avgdigamma_y = 0.0;

        for i in 0..x.len() {
            let eps = joint_knn[i];

            // Count X-neighbors within distance eps
            let mut nx = 0;
            for j in 0..x.len() {
                if x_dists[i][j] <= eps + 1e-9 {
                    nx += 1;
                }
            }

            // Count Y-neighbors within distance eps
            let mut ny = 0;
            for j in 0..y.len() {
                if y_dists[i][j] <= eps + 1e-9 {
                    ny += 1;
                }
            }

            if nx > 0 && ny > 0 {
                avgdigamma_x += knn_estimators::digamma(nx as f64);
                avgdigamma_y += knn_estimators::digamma(ny as f64);
            }
        }

        avgdigamma_x /= n;
        avgdigamma_y /= n;

        // Kraskov MI: digamma(k) - <digamma(nx)> - <digamma(ny)> + digamma(n)
        let mi_nats = knn_estimators::digamma(k as f64) - avgdigamma_x - avgdigamma_y
            + knn_estimators::digamma(n);

        mi_nats / std::f64::consts::LN_2 // Convert to bits
    }

    /// Conditional mutual information I(X;Y|Z) for continuous data via k-NN
    pub fn continuous_conditional_knn(
        x: &[Vec<f32>],
        y: &[Vec<f32>],
        z: &[Vec<f32>],
        k: usize,
    ) -> f64 {
        assert_eq!(x.len(), y.len(), "X and Y must have same length");
        assert_eq!(x.len(), z.len(), "X and Z must have same length");

        // I(X;Y|Z) = I(X;Y,Z) - I(X;Z)
        let mut yz = vec![];
        for (yi, zi) in y.iter().zip(z.iter()) {
            let mut combined = yi.clone();
            combined.extend(zi.iter());
            yz.push(combined);
        }

        let i_xyz = Self::continuous_knn(x, &yz, k);
        let i_xz = Self::continuous_knn(x, z, k);

        (i_xyz - i_xz).max(0.0)
    }
}
