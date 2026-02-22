# Information

Information theory metrics for measuring agent-environment coupling.

## Core Modules

`measures/shannon.rs` - Shannon entropy (discrete & continuous k-NN)
- `Shannon::entropy()` - discrete entropy H(X)
- `Shannon::continuous_entropy()` - k-NN based entropy (Kraskov et al. 2004)

`measures/mutual/calculation.rs` - Mutual information I(X;Y)
- `MutualInfo::discrete()` - discrete MI from joint distributions
- `MutualInfo::continuous_knn()` - Kraskov k-NN continuous MI
- `MutualInfo::continuous_conditional_knn()` - conditional MI I(X;Y|Z)

`measures/knn_estimators.rs` (internal) - k-NN distance calculations, digamma function

## Quick Example

```rust
use information::measures::mutual::MutualInfo;

let agent_state = vec![vec![1.0], vec![2.0], vec![3.0]];
let environment = vec![vec![1.1], vec![2.1], vec![3.1]];

// How many bits does agent share with environment?
let bits = MutualInfo::continuous_knn(&agent_state, &environment, k=3);
```

## Design

All methods return entropy in **bits** (standard units). Algorithm is Kraskov et al. (2004), no external dependencies, pure Rust.

Fractals module (L-systems) is scaffolded; may be replaced with reaction-diffusion model.