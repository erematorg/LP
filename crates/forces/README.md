# Forces

Gravitational mechanics and Newton's laws for physics-based force systems.

## Scope and units
- Uses SI-style units (meters, seconds, Newtons) for mass/force/velocity.
- Applies forces and integrates velocities explicitly; no global momentum/energy reconciliation yet.
- Gravity supports uniform fields and n-body/Barnes–Hut, with configurable softening/theta.
- Integration uses variable `Time.delta_secs()` (no fixed physics tick yet); dual-clock (physics vs diurnal/biology) is not implemented.

## Scope & Limits
- LP-0 integrates F = ma with explicit/symplectic Euler and optional acceleration clamps for stability; this is a numerical method, not a physical law.
- Gravity defaults to a sim-tuned constant and softened inverse-square forces; Barnes-Hut is an approximation for large N.
- Mutual gravity mode is exact pairwise O(N^2); treat ~100 active sources as the LP-0 realtime comfort range.
- Potential energy and work accounting are not yet tracked; conservation diagnostics are partial.

## Conservation status
- Linear momentum is computable via `calculate_momentum()` but **not enforced globally**.
- **Angular momentum is NOT conserved**; `Velocity.angvel` exists but no torque/conservation tracking yet.
- **Mass is NOT globally conserved** in current implementation (per lp-physics-chem-invariants contracts).

## Material responses (not in this crate)
- **Friction, contact forces, elasticity, plasticity, viscosity**: These are material behaviors, not fundamental force laws.
- Deferred to matter/MPM coupling (see tmp/GD/Systems_Overview.md line 13-16).
- Forces crate covers interaction laws (gravity, F=ma); material response belongs in matter.
