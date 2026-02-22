/// K-NN based continuous entropy/MI estimators (ported from NPEET)
/// No external dependencies — pure Rust port of Kraskov et al. 2004
/// Reference: https://link.aps.org/doi/10.1103/PhysRevE.69.066138
use std::cmp::Ordering;

/// Build simple k-NN tree using distance-sorted search (brute force for <10k points)
/// For larger datasets, could replace with actual spatial tree
pub fn build_distance_matrix(points: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let n = points.len();
    let mut distances = vec![vec![f32::INFINITY; n]; n];

    for i in 0..n {
        for j in i + 1..n {
            let dist = euclidean_distance(&points[i], &points[j]);
            distances[i][j] = dist;
            distances[j][i] = dist;
        }
    }

    distances
}

/// Euclidean distance between two points
fn euclidean_distance(p1: &[f32], p2: &[f32]) -> f32 {
    assert_eq!(p1.len(), p2.len(), "Points must have same dimension");
    p1.iter()
        .zip(p2.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Query k-nearest neighbors for each point
/// Returns vector of distances to k-th neighbor for each point
pub fn knn_distances(distance_matrix: &[Vec<f32>], k: usize) -> Vec<f32> {
    let n = distance_matrix.len();
    assert!(k < n, "k must be smaller than number of points");

    let mut knn_dists = vec![0.0; n];

    for i in 0..n {
        // Skip self (distance 0), sort rest
        let mut dists: Vec<f32> = distance_matrix[i]
            .iter()
            .enumerate()
            .filter(|(j, _)| i != *j)
            .map(|(_, &d)| d)
            .collect();

        dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        if k < dists.len() {
            knn_dists[i] = dists[k - 1]; // k-th nearest neighbor distance
        }
    }

    knn_dists
}

/// Add small noise to avoid exact duplicates (NPEET technique)
pub fn add_noise(data: &[Vec<f32>], noise_scale: f32) -> Vec<Vec<f32>> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    data.iter()
        .enumerate()
        .map(|(i, point)| {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_usize(i);
            let hash_val = hasher.finish() as f32 / (u64::MAX as f32);

            point
                .iter()
                .enumerate()
                .map(|(j, &v)| v + noise_scale * (hash_val * ((j + 1) as f32)).sin())
                .collect()
        })
        .collect()
}

/// Digamma function approximation (Euler-Mascheroni constant + harmonic series)
/// Used in entropy estimation following Kraskov et al.
pub fn digamma(x: f64) -> f64 {
    if x < 8.0 {
        // Recursion for small x
        digamma(x + 1.0) - 1.0 / x
    } else {
        // Stirling asymptotic expansion: ψ(x) = ln(x) - 1/(2x) - 1/(12x²) + 1/(120x⁴) - ...
        // Ref: DLMF 5.11.2 — accurate to <1e-7 for x >= 8
        let inv_x = 1.0 / x;
        let inv_x2 = inv_x * inv_x;
        x.ln() - 0.5 * inv_x - inv_x2 / 12.0 + inv_x2 * inv_x2 / 120.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let p1 = vec![0.0, 0.0];
        let p2 = vec![3.0, 4.0];
        assert!((euclidean_distance(&p1, &p2) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_digamma_convergence() {
        let d5 = digamma(5.0);
        let d6 = digamma(6.0);
        // digamma(n+1) = digamma(n) + 1/n (recurrence relation)
        assert!((d6 - d5 - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_digamma_known_values() {
        // Ground-truth tabulated values (DLMF 5.4.14):
        // digamma(1) = -γ where γ = 0.5772156649... (Euler-Mascheroni constant)
        const EULER_MASCHERONI: f64 = 0.5772156649015328;
        let d1 = digamma(1.0);
        assert!(
            (d1 - (-EULER_MASCHERONI)).abs() < 1e-5,
            "digamma(1) should be -γ ≈ -0.5772, got {}",
            d1
        );

        // digamma(2) = 1 - γ
        let d2 = digamma(2.0);
        assert!(
            (d2 - (1.0 - EULER_MASCHERONI)).abs() < 1e-5,
            "digamma(2) should be 1-γ ≈ 0.4228, got {}",
            d2
        );

        // digamma(1/2) = -γ - 2*ln(2) ≈ -1.9635
        let d_half = digamma(0.5);
        let expected_half = -EULER_MASCHERONI - 2.0 * std::f64::consts::LN_2;
        assert!(
            (d_half - expected_half).abs() < 1e-4,
            "digamma(0.5) should be ≈ -1.9635, got {}",
            d_half
        );
    }
}
