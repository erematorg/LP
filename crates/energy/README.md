# Energy

Energy conservation, thermodynamics, electromagnetism, and wave mechanics for LP physics simulations.

## Scope and units
- Tracks energy in joules via an accounting ledger; conservation is enforced where modeled.
- Thermodynamics, EM, and waves modules are present but remain partial implementations.
- Uses consistent sim units (SI-style); couple to time/space steps used by the broader sim.

## Scope & Limits
- LP-0 thermodynamics and EM use explicit approximations: pairwise interactions, cutoffs, and quasi-static assumptions for performance.
- Wave solvers use finite differences and simplified damping models; energy coupling is partial.
- Thermal energy uses constant heat capacity and clamps temperatures at 0 K; phase change and EOS are deferred.

## Not yet implemented
- **Wave/EM energy accounting**: Wave damping exists but not coupled to energy ledger; EM fields exist but no work/energy tracking.
- **EOS, convection, latent heat, phase transitions**: Deferred to matter/MPM coupling (see tmp/GD/Systems_Overview.md).
