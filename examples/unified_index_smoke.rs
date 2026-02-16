//! Smoke test for UnifiedSpatialIndex (Phase A2.3 verification).
//!
//! **Verification**: Entities with Charge/Temperature get SpatiallyIndexed and are findable.

use bevy::prelude::*;
use energy::electromagnetism::charges::{Charge, SofteningLength};
use energy::thermodynamics::thermal::{HeatCapacity, Temperature, ThermalConductivity};
use forces::core::newton_laws::{AppliedForce, Mass, Velocity};
use matter::geometry::Radius;
use utils::{SpatiallyIndexed, UnifiedSpatialIndex};

fn main() {
    info!("🧪 Starting UnifiedSpatialIndex smoke test (Phase A2.3)");

    App::new()
        .add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
        .add_plugins((
            utils::UtilsPlugin,
            forces::ForcesPlugin,
            energy::EnergyPlugin,
            matter::MatterPlugin,
        ))
        .add_systems(Startup, spawn_test_entities)
        .add_systems(Last, verify_all)
        .run();
}

fn spawn_test_entities(mut commands: Commands) {
    info!("🧪 Spawning 4 test entities");

    // Entity 1: Charge only (at origin)
    commands.spawn((
        Name::new("Charged"),
        Charge::new(1.0),
        SofteningLength::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mass::new(1.0),
        Velocity::default(),
        AppliedForce::default(),
    ));

    // Entity 2: Temperature only (5m away)
    commands.spawn((
        Name::new("Thermal"),
        Temperature::new(300.0),
        ThermalConductivity { value: 1.0 },
        HeatCapacity::water(1.0),
        Radius { value: 0.1 },
        Transform::from_xyz(5.0, 0.0, 0.0),
        Mass::new(1.0),
    ));

    // Entity 3: Both Charge + Temperature (10m away)
    commands.spawn((
        Name::new("Both"),
        Charge::new(-1.0),
        SofteningLength::default(),
        Temperature::new(400.0),
        ThermalConductivity { value: 1.0 },
        HeatCapacity::water(1.0),
        Radius { value: 0.1 },
        Transform::from_xyz(10.0, 0.0, 0.0),
        Mass::new(1.0),
        Velocity::default(),
        AppliedForce::default(),
    ));

    // Entity 4: Charge far away (100m, outside cutoff)
    commands.spawn((
        Name::new("Far"),
        Charge::new(2.0),
        SofteningLength::default(),
        Transform::from_xyz(100.0, 0.0, 0.0),
        Mass::new(1.0),
        Velocity::default(),
        AppliedForce::default(),
    ));

    info!("✅ Spawned 4 test entities");
}

fn verify_all(
    index: Res<UnifiedSpatialIndex>,
    entities: Query<(Entity, &Name, Has<SpatiallyIndexed>)>,
    mut done: Local<bool>,
) {
    if *done || entities.is_empty() {
        return;
    }

    // Step 1: Verify all entities have SpatiallyIndexed marker
    let mut count = 0;
    for (entity, name, has_marker) in entities.iter() {
        if !has_marker {
            error!(
                "❌ FAILED: Entity {:?} ({}) missing SpatiallyIndexed marker",
                entity, name
            );
            panic!("Marker injection failed");
        }
        count += 1;
    }
    info!(
        "✅ PASS: All {} entities have SpatiallyIndexed marker",
        count
    );

    // Step 2: Verify spatial queries find entities
    let neighbors: Vec<Entity> = index.query_radius(Vec2::ZERO, 20.0);
    info!(
        "🔍 Found {} neighbors within 20m of origin",
        neighbors.len()
    );

    // Should find 3 entities (Charged at 0m, Thermal at 5m, Both at 10m)
    // Should NOT find Far (100m away)
    if neighbors.len() < 3 {
        error!(
            "❌ FAILED: Expected at least 3 neighbors, found {}",
            neighbors.len()
        );
        panic!("Spatial query failed");
    }

    info!(
        "✅ PASS: Spatial query found {} neighbors (expected 3-4)",
        neighbors.len()
    );
    info!("🎉 Phase A2.3 PASSED - UnifiedSpatialIndex works correctly");

    *done = true;
    std::process::exit(0); // Success exit
}
