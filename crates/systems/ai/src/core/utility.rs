use crate::prelude::*;
use bevy::prelude::*;
use rand::prelude::*; //still plan to use bevy_rand instead
use std::collections::HashMap;

/// Represents a utility score for decision-making
/// Normalized between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Component, Reflect)]
pub struct UtilityScore(f32);

impl UtilityScore {
    /// Create a new utility score, clamping to valid range
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw score value
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Normalize a collection of scores to sum to 1.0
    pub fn normalize_scores(scores: &mut [UtilityScore]) {
        let total: f32 = scores.iter().map(|score| score.value()).sum();
        if total > 0.0 {
            for score in scores.iter_mut() {
                *score = UtilityScore::new(score.value() / total);
            }
        } else {
            // If all scores are 0, distribute evenly
            let equal_value = if scores.is_empty() {
                0.0
            } else {
                1.0 / scores.len() as f32
            };
            for score in scores.iter_mut() {
                *score = UtilityScore::new(equal_value);
            }
        }
    }

    /// Multiply the score by a factor (adjusts importance)
    pub fn multiply_by_factor(&self, factor: f32) -> Self {
        Self::new(self.0 * factor)
    }

    /// Combine two scores with AND logic (both must be true)
    /// Returns lower values as both scores must be high
    pub fn and_with(&self, other: &Self) -> Self {
        Self::new(self.0 * other.0)
    }

    /// Blend two scores with custom importance weights
    /// weight_a and weight_b should ideally sum to 1.0
    pub fn blend_weighted(&self, other: &Self, weight_a: f32, weight_b: f32) -> Self {
        Self::new(self.0 * weight_a + other.0 * weight_b)
    }

    /// Combine scores with OR logic (either can be true)
    /// P(A or B) = P(A) + P(B) - P(A and B)
    pub fn or_with(&self, other: &Self) -> Self {
        Self::new(self.0 + other.0 - (self.0 * other.0))
    }

    /// Get the opposite score (1 - score)
    /// Useful for negating or inverting priorities
    pub fn opposite(&self) -> Self {
        Self::new(1.0 - self.0)
    }

    /// Perform weighted random selection between multiple options
    pub fn weighted_select<T: Clone, R: Rng>(
        options: &[(T, UtilityScore)],
        rng: &mut R,
    ) -> Option<T> {
        let total_weight: f32 = options.iter().map(|(_, score)| score.value()).sum();
        if total_weight <= 0.0 || options.is_empty() {
            return None;
        }
        let random_point = rng.random_range(0.0..total_weight);
        let mut cumulative_weight = 0.0;
        for (option, score) in options {
            cumulative_weight += score.value();
            if random_point <= cumulative_weight {
                return Some(option.clone());
            }
        }
        // Fallback to last option (shouldn't happen with proper weights)
        options.last().map(|(opt, _)| opt.clone())
    }
}

/// Possible AI behaviors that can be selected based on utility scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Behavior {
    Idle,      // Default state, minimal activity
    Hunt,      // Pursuing prey or resource
    Flee,      // Escaping from threat
    Explore,   // Discovering new areas
    Fight,     // Engaging in combat
    Rest,      // Recovering energy
    Socialize, // Interacting with others
}

/// Selects the most appropriate behavior based on module utility scores
pub fn determine_behavior(
    modules: &[(&dyn AIModule, UtilityScore, Behavior)],
) -> (Behavior, UtilityScore) {
    if modules.is_empty() {
        return (Behavior::Idle, UtilityScore::new(0.0));
    }

    // Create normalized version of the scores for better decision making
    let mut normalized_scores: Vec<UtilityScore> =
        modules.iter().map(|(_, score, _)| *score).collect();
    UtilityScore::normalize_scores(&mut normalized_scores);

    // Find behavior with highest normalized score
    let mut best_index = 0;
    let mut best_score = normalized_scores[0];

    for (i, score) in normalized_scores.iter().enumerate().skip(1) {
        if score > &best_score {
            best_score = *score;
            best_index = i;
        }
    }

    // Return the original behavior and original score (not normalized)
    // This preserves the absolute utility value while making selection based on normalized scores
    let (_, original_score, behavior) = modules[best_index];
    (behavior, original_score)
}

/// A cached result with timestamp for expiration
#[derive(Clone, Debug)]
struct CachedValue {
    value: UtilityScore,
    timestamp: f32,
}

/// Global utility cache resource
#[derive(Resource)]
pub struct UtilityCache {
    cache: HashMap<String, CachedValue>,
    ttl: f32,
}

impl Default for UtilityCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            ttl: 0.5, // Default 0.5 seconds TTL
        }
    }
}

impl UtilityCache {
    pub fn new(ttl: f32) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }

    /// Get cached value if not expired
    pub fn get(&self, key: &str, current_time: f32) -> Option<UtilityScore> {
        self.cache.get(key).and_then(|cached| {
            if current_time - cached.timestamp < self.ttl {
                Some(cached.value)
            } else {
                None
            }
        })
    }

    /// Store value with timestamp
    pub fn insert(&mut self, key: String, value: UtilityScore, current_time: f32) {
        self.cache.insert(
            key,
            CachedValue {
                value,
                timestamp: current_time,
            },
        );
    }

    /// Get value if cached, otherwise calculate and store
    pub fn get_or_insert_with<F>(
        &mut self,
        key: String,
        calculator: F,
        current_time: f32,
    ) -> UtilityScore
    where
        F: FnOnce() -> UtilityScore,
    {
        if let Some(value) = self.get(&key, current_time) {
            return value;
        }

        let value = calculator();
        self.insert(key, value, current_time);
        value
    }

    /// Clean up expired entries
    pub fn cleanup(&mut self, current_time: f32) {
        self.cache
            .retain(|_, cached| current_time - cached.timestamp < self.ttl);
    }
}

/// Entity-specific cache component for better scaling
#[derive(Component, Default)]
pub struct EntityUtilityCache {
    cache: HashMap<String, CachedValue>,
}

impl EntityUtilityCache {
    /// Get cached value if not expired
    pub fn get(&self, key: &str, current_time: f32, ttl: f32) -> Option<UtilityScore> {
        self.cache.get(key).and_then(|cached| {
            if current_time - cached.timestamp < ttl {
                Some(cached.value)
            } else {
                None
            }
        })
    }

    /// Store value with timestamp
    pub fn insert(&mut self, key: String, value: UtilityScore, current_time: f32) {
        self.cache.insert(
            key,
            CachedValue {
                value,
                timestamp: current_time,
            },
        );
    }

    /// Clean up expired entries
    pub fn cleanup(&mut self, current_time: f32, ttl: f32) {
        self.cache
            .retain(|_, cached| current_time - cached.timestamp < ttl);
    }
}

/// Trait for AIModules that support caching
pub trait CacheableModule: AIModule {
    /// Generate a cache key for this module
    fn cache_key(&self) -> Option<String> {
        None // Default implementation returns None (no caching)
    }

    /// Get utility with caching if available
    fn cached_utility(
        &self,
        entity_cache: Option<&mut EntityUtilityCache>,
        global_cache: Option<&mut UtilityCache>,
        current_time: f32,
    ) -> UtilityScore {
        // Try to get a cache key
        let Some(key) = self.cache_key() else {
            return self.utility(); // No key means no caching
        };

        // Try entity cache first (better locality and parallelism)
        if let Some(entity_cache) = entity_cache {
            let ttl = global_cache.as_ref().map(|c| c.ttl).unwrap_or(0.5);

            if let Some(value) = entity_cache.get(&key, current_time, ttl) {
                return value;
            }

            // Not in entity cache, calculate and store
            let value = self.utility();
            entity_cache.insert(key.clone(), value, current_time);

            // Also update global cache if available
            if let Some(global_cache) = global_cache {
                global_cache.insert(key, value, current_time);
            }

            return value;
        }

        // No entity cache, try global cache
        if let Some(global_cache) = global_cache {
            return global_cache.get_or_insert_with(key, || self.utility(), current_time);
        }

        // No caching available, calculate directly
        self.utility()
    }
}

/// System that periodically cleans up the utility cache
pub fn cleanup_utility_cache_system(
    time: Res<Time>,
    mut global_cache: ResMut<UtilityCache>,
    mut entity_caches: Query<&mut EntityUtilityCache>,
) {
    let current_time = time.elapsed_secs_f64() as f32;

    // Clean up global cache
    global_cache.cleanup(current_time);

    // Clean up entity caches
    for mut entity_cache in &mut entity_caches {
        entity_cache.cleanup(current_time, global_cache.ttl);
    }
}

/// Helper function to get current time as f32
pub fn get_current_time(time: &Time) -> f32 {
    time.elapsed_secs_f64() as f32
}

/// Simple function to initialize the caching system
pub fn setup_utility_caching(app: &mut App, ttl: f32) {
    app.insert_resource(UtilityCache::new(ttl))
        .add_systems(Last, cleanup_utility_cache_system);
}
