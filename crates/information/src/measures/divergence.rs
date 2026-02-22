/// Kullback-Leibler divergence and related measures.
/// Core information-theoretic distance metrics for comparing probability distributions.
///
/// LP usage: measure evolutionary distance between species, compare creature behaviour
/// distributions, quantify trait drift between populations. In multiplayer, these
/// metrics let the simulation decide when two players' lineages count as separate species.
pub struct KLDivergence;

impl KLDivergence {
    /// D(P||Q) = Σ P(i) log₂(P(i)/Q(i))
    ///
    /// How many extra bits are needed to encode P-distributed events using a code
    /// optimised for Q. NOT symmetric. Returns +∞ when P has mass where Q has none.
    pub fn divergence(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        let mut kl = 0.0;
        for (&p, &q) in p_probs.iter().zip(q_probs) {
            if p > 0.0 && q > 0.0 {
                kl += p * (p / q).log2();
            } else if p > 0.0 {
                // P has support where Q does not — infinite divergence by definition
                return f64::INFINITY;
            }
            // p == 0 contributes 0 regardless of q  (0 · log(0/q) := 0)
        }
        kl
    }

    /// JS(P,Q) = 0.5 · [D(P||M) + D(Q||M)]  where M = 0.5·(P+Q)
    ///
    /// Symmetric, always finite, bounded [0, 1] bits (log₂ base).
    /// Reaches 1 bit only when P and Q have fully disjoint support.
    /// Preferred over raw KL for comparing creature trait distributions
    /// because it never blows up and is a proper metric (square root is a distance).
    pub fn jensen_shannon(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        let m: Vec<f64> = p_probs
            .iter()
            .zip(q_probs)
            .map(|(&p, &q)| 0.5 * (p + q))
            .collect();

        // M[i] >= 0.5 · max(P[i], Q[i]), so KL(P||M) and KL(Q||M) are always finite.
        0.5 * (Self::divergence(p_probs, &m) + Self::divergence(q_probs, &m))
    }

    /// H(P,Q) = -Σ P(i) log₂(Q(i))
    ///
    /// Cross-entropy: bits to encode P-distributed events with a Q-optimal code.
    /// H(P,Q) = H(P) + D(P||Q). Returns +∞ when Q has no mass where P has mass.
    pub fn cross_entropy(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        let mut ce = 0.0;
        for (&p, &q) in p_probs.iter().zip(q_probs) {
            if p > 0.0 && q > 0.0 {
                ce -= p * q.log2();
            } else if p > 0.0 {
                return f64::INFINITY;
            }
        }
        ce
    }

    /// TV(P,Q) = 0.5 · Σ|P(i) - Q(i)|
    ///
    /// Total variation distance: the maximum probability gap any single event
    /// can have between P and Q. Symmetric, bounded [0, 1], no log required.
    /// Fastest way to ask "how different are these two distributions?" when you
    /// do not need the information-theoretic interpretation of KL/JS.
    pub fn total_variation(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );
        0.5 * p_probs
            .iter()
            .zip(q_probs)
            .map(|(&p, &q)| (p - q).abs())
            .sum::<f64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kl_identical_is_zero() {
        let p = [0.5, 0.5];
        assert!(KLDivergence::divergence(&p, &p).abs() < 1e-10);
    }

    #[test]
    fn kl_disjoint_support_is_infinity() {
        let p = [1.0, 0.0];
        let q = [0.0, 1.0];
        assert_eq!(KLDivergence::divergence(&p, &q), f64::INFINITY);
    }

    #[test]
    fn kl_known_value() {
        // D([0.5, 0.5] || [0.25, 0.75]) = 0.5*log2(2) + 0.5*log2(2/3) ≈ 0.2075
        let p = [0.5, 0.5];
        let q = [0.25, 0.75];
        let kl = KLDivergence::divergence(&p, &q);
        let expected = 0.5 * (0.5_f64 / 0.25).log2() + 0.5 * (0.5_f64 / 0.75).log2();
        assert!((kl - expected).abs() < 1e-10, "got {}", kl);
    }

    #[test]
    fn js_identical_is_zero() {
        let p = [0.25, 0.25, 0.25, 0.25];
        assert!(KLDivergence::jensen_shannon(&p, &p).abs() < 1e-10);
    }

    #[test]
    fn js_disjoint_support_is_one_bit() {
        let p = [1.0, 0.0];
        let q = [0.0, 1.0];
        let js = KLDivergence::jensen_shannon(&p, &q);
        assert!((js - 1.0).abs() < 1e-10, "expected 1 bit, got {}", js);
    }

    #[test]
    fn js_is_symmetric() {
        let p = [0.7, 0.2, 0.1];
        let q = [0.1, 0.5, 0.4];
        let diff =
            (KLDivergence::jensen_shannon(&p, &q) - KLDivergence::jensen_shannon(&q, &p)).abs();
        assert!(diff < 1e-12, "JS must be symmetric, diff = {}", diff);
    }

    #[test]
    fn js_bounded_between_zero_and_one() {
        let p = [0.6, 0.3, 0.1];
        let q = [0.1, 0.2, 0.7];
        let js = KLDivergence::jensen_shannon(&p, &q);
        assert!(js >= 0.0 && js <= 1.0, "JS out of [0,1]: {}", js);
    }

    #[test]
    fn cross_entropy_of_self_equals_entropy() {
        // H(P, P) = H(P). For fair coin P=[0.5,0.5], H = 1 bit.
        let p = [0.5, 0.5];
        let h = KLDivergence::cross_entropy(&p, &p);
        assert!((h - 1.0).abs() < 1e-10, "expected 1 bit, got {}", h);
    }

    #[test]
    fn total_variation_identical_is_zero() {
        let p = [0.3, 0.3, 0.4];
        assert!(KLDivergence::total_variation(&p, &p).abs() < 1e-12);
    }

    #[test]
    fn total_variation_disjoint_is_one() {
        let p = [1.0, 0.0];
        let q = [0.0, 1.0];
        assert!((KLDivergence::total_variation(&p, &q) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn total_variation_bounded() {
        let p = [0.6, 0.4];
        let q = [0.2, 0.8];
        let tv = KLDivergence::total_variation(&p, &q);
        assert!(tv >= 0.0 && tv <= 1.0, "TV must be in [0, 1], got {}", tv);
    }
}
