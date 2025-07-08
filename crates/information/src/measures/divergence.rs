/// Kullback-Leibler divergence and related measures
/// Core information-theoretic distance metrics
pub struct KLDivergence;

impl KLDivergence {
    /// Calculate KL divergence D(P||Q) = Σ P(i) log₂(P(i)/Q(i))
    /// Measures how distribution P differs from reference Q
    /// Returns divergence in bits - NOT symmetric
    pub fn divergence(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        let mut kl_div = 0.0;
        for (&p, &q) in p_probs.iter().zip(q_probs) {
            if p > 0.0 && q > 0.0 {
                kl_div += p * (p / q).log2();
            } else if p > 0.0 && q == 0.0 {
                // P has probability where Q doesn't - infinite divergence
                return f64::INFINITY;
            }
            // p == 0.0 contributes nothing to KL divergence
        }

        kl_div
    }

    /// Calculate Jensen-Shannon divergence - symmetric version of KL
    /// JS(P,Q) = 0.5 * [D(P||M) + D(Q||M)] where M = 0.5*(P+Q)
    /// Always finite and bounded [0, 1] bits
    pub fn jensen_shannon(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        // Calculate mixture distribution M = 0.5*(P+Q)
        let m_probs: Vec<f64> = p_probs
            .iter()
            .zip(q_probs)
            .map(|(&p, &q)| 0.5 * (p + q))
            .collect();

        let kl_pm = Self::divergence(p_probs, &m_probs);
        let kl_qm = Self::divergence(q_probs, &m_probs);

        0.5 * (kl_pm + kl_qm)
    }

    /// Calculate cross-entropy H(P,Q) = -Σ P(i) log₂(Q(i))  
    /// Useful for ML loss functions and distribution comparison
    pub fn cross_entropy(p_probs: &[f64], q_probs: &[f64]) -> f64 {
        assert_eq!(
            p_probs.len(),
            q_probs.len(),
            "P and Q must have same length"
        );

        let mut cross_ent = 0.0;
        for (&p, &q) in p_probs.iter().zip(q_probs) {
            if p > 0.0 && q > 0.0 {
                cross_ent -= p * q.log2();
            } else if p > 0.0 && q == 0.0 {
                return f64::INFINITY;
            }
        }

        cross_ent
    }
}
