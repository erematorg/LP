# Energy

Energy conservation, thermodynamics, electromagnetism, and wave mechanics for LP physics simulations.

## Scope and units
- Tracks energy in joules via an accounting ledger; conservation is enforced where modeled.
- Thermodynamics, EM, and waves modules are present but remain partial scaffolds.
- Uses consistent sim units (SI-style); couple to time/space steps used by the broader sim.

## Not yet implemented
- **Wave/EM energy accounting**: Wave damping exists but not coupled to energy ledger; EM fields exist but no work/energy tracking.
- **EOS, convection, latent heat, phase transitions**: Deferred to matter/MPM coupling (see tmp/GD/Systems_Overview.md).
