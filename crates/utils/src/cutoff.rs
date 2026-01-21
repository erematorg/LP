//! Smooth cutoff function for pair interactions.
//!
//! C¹ continuous force-switch (cubic spline) from GROMACS/LAMMPS.
//! Ensures forces → 0 smoothly at r_cut (no discontinuity).

/// C¹ continuous force-switch (cubic spline).
///
/// Returns multiplicative factor S(r) where Force = F_bare * S(r).
/// Smoothly transitions from 1.0 (full force) to 0.0 (no force) between r_on and r_cut.
///
/// Formula: S(x) = 1 - 3x² + 2x³ where x = (r - r_on) / (r_cut - r_on)
///
/// # Arguments
/// * `r` - Distance between entities
/// * `r_on` - Start of transition (typically 0.8 * r_cut)
/// * `r_cut` - Cutoff radius (force = 0 beyond this)
///
/// # Returns
/// Multiplicative factor in [0.0, 1.0]
pub fn force_switch(r: f32, r_on: f32, r_cut: f32) -> f32 {
    if r >= r_cut {
        return 0.0;
    }
    if r <= r_on {
        return 1.0;
    }

    // Cubic spline interpolation
    let x = (r - r_on) / (r_cut - r_on);
    1.0 - 3.0 * x.powi(2) + 2.0 * x.powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_switch() {
        let r_on = 8.0;
        let r_cut = 10.0;

        // Full force before r_on
        assert_eq!(force_switch(7.0, r_on, r_cut), 1.0);

        // No force after r_cut
        assert_eq!(force_switch(10.0, r_on, r_cut), 0.0);
        assert_eq!(force_switch(11.0, r_on, r_cut), 0.0);

        // Smooth transition in between
        let mid = (r_on + r_cut) / 2.0;
        let factor = force_switch(mid, r_on, r_cut);
        assert!(factor > 0.0 && factor < 1.0);
    }
}
