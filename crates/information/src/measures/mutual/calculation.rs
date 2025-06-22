use std::collections::HashMap;

/// Core mutual information calculation for discrete and continuous variables
/// 
/// Mutual Information I(X;Y) measures how much knowing X tells us about Y
/// Formula: I(X;Y) = H(X) + H(Y) - H(X,Y)
pub struct MutualInfo;

impl MutualInfo {
    /// Calculate mutual information for discrete variables
    /// Returns I(X;Y) in bits
    pub fn discrete(x_values: &[i32], y_values: &[i32]) -> f64 {
        assert_eq!(x_values.len(), y_values.len(), "X and Y must have same length");
        
        let n = x_values.len() as f64;
        if n == 0.0 { return 0.0; }
        
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
            let p_x = *x_counts.get(&x).unwrap() as f64 / n;
            let p_y = *y_counts.get(&y).unwrap() as f64 / n;
            
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
        assert_eq!(x_values.len(), y_values.len(), "X and Y must have same length");
        
        if x_values.is_empty() || bins == 0 { return 0.0; }
        
        // Find ranges for binning
        let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Handle edge case where all values are the same
        if x_min == x_max || y_min == y_max { return 0.0; }
        
        let x_step = (x_max - x_min) / bins as f64;
        let y_step = (y_max - y_min) / bins as f64;
        
        // Convert to discrete bins
        let x_bins: Vec<i32> = x_values.iter()
            .map(|&x| ((x - x_min) / x_step).floor().min(bins as f64 - 1.0) as i32)
            .collect();
        let y_bins: Vec<i32> = y_values.iter()
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
        if n <= 1.0 { return 0.0; }
        
        // Count unique joint states for Miller-Madow correction
        let joint_unique = x_values.iter()
            .zip(y_values)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;
        
        // Miller-Madow bias correction: (K-1)/(2N) where K is unique states
        let bias_correction = (joint_unique - 1.0) / (2.0 * n);
        (raw_mi - bias_correction).max(0.0)
    }
    
}