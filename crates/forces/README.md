# Forces

Gravitational mechanics and Newton's laws for physics-based force systems.

## Core Principles

- Uses SI-style units (meters, seconds, Newtons) for mass/force/velocity.
- Applies forces and integrates velocities explicitly; no global momentum/energy reconciliation yet.
- Gravity supports uniform fields and n-body mutual gravity, with configurable softening.
- Integration uses variable `Time.delta_secs()` (no fixed physics tick yet); dual-clock (physics vs diurnal/biology) is not implemented.

## Scope & Limits

- LP-0 integrates F = ma with explicit/symplectic Euler (1st order) and optional acceleration clamps for stability; this is a numerical method, not a physical law.
- Gravity defaults to a sim-tuned constant and softened inverse-square forces (Plummer softening: F = GMm·r/(r²+ε²)^1.5).
- Mutual gravity mode is exact pairwise O(N²); treat ~100 active sources as the LP-0 realtime comfort range.
- Linear momentum is computable but **not enforced globally**. Angular momentum and mass conservation are **not yet tracked**.
- **Contact forces, friction, elasticity, plasticity, viscosity**: Material behaviors, deferred to matter/MPM coupling.
- Potential energy and work accounting are partial; conservation diagnostics incomplete.

## Status

Production-ready for gravity and Coulomb forces at N~100. Known limitation: 1st-order integrator causes ~0.1% energy drift over long orbits; upgrade to Velocity Verlet (2nd order) planned. Coulomb singularity at r<softening is handled with per-charge multiplier (default 0.0). Contact physics not yet implemented.
