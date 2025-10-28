# Systems

The `systems` workspace crate bundles LP's high-level simulation layers
(decision-making, acoustics, future MPM integration, and persistence) so they
can be added to a Bevy app with a single plugin. It sits on top of the domain
crates (`energy`, `forces`, `information`, `matter`) and takes care of wiring and
scheduling.

## Modules

- `ai` - utility-driven agency used by LP's creatures and actors.
- `acoustics` - scaffold for physics-based sound that will hook into matter and
  wave simulation.
- `mpm` - placeholder for the upcoming Material Point Method solver.
- `save_system` - shared save / load infrastructure.

Each module exposes a `prelude` for selective use, while
`systems::SystemsPlugin` pulls everything together.

## Quick start

```rust
use bevy::prelude::*;
use systems::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SystemsPlugin::default())
        .run();
}
```
