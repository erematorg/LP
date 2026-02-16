//! N-Body Gravity + Electromagnetism — Enhanced Physics Sandbox
//!
//! Visual: orbital trails, auto-zoom camera, starfield background.
//! Physics: Plummer-softened gravity + Coulomb electrostatic forces
//! Features: charge interactions, energy/momentum conservation tracking
//! Controls: comprehensive egui panel for all physics parameters
//!
//! Run: `cargo run --example basic_forces`

use bevy::{color::palettes::css::*, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use energy::EnergyPlugin;
use energy::prelude::*;
use forces::PhysicsSet;
use forces::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use utils::UtilsPlugin;

const SOFTENING: f32 = 8.0;
const TRAIL_LIFETIME: f32 = 3.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "N-Body Gravity — Mini Solar System".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            UtilsPlugin,
            NewtonLawsPlugin,
            GravityPlugin::new(),
            EnergyPlugin, // Includes Electromagnetism + Thermodynamics + Conservation + Waves
            EguiPlugin::default(),
        ))
        .insert_resource(UniformGravity {
            acceleration: Vec3::ZERO,
        })
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
        .insert_resource(GravityParams::default().with_softening(SOFTENING))
        .init_resource::<EnergyBaseline>()
        .insert_resource(DiagTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(TrailTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .init_resource::<SimulationControls>()
        .add_systems(Startup, (setup, spawn_starfield, log_initial_velocities))
        .add_systems(EguiPrimaryContextPass, egui_controls)
        .add_systems(
            Update,
            (
                apply_controls,
                spawn_trails,
                fade_trails,
                (camera_follow, physics_diagnostics).after(PhysicsSet::Integrate),
            ),
        )
        .run();
}

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
struct CelestialBody {
    radius: f32,
}

#[derive(Component)]
struct Star;

#[derive(Component)]
struct OrbitalRef {
    initial_dist: f32,
    initial_speed: f32,
    period: f32,
}

#[derive(Component)]
struct TrailDot {
    lifetime: f32,
    max_lifetime: f32,
    color: Color,
}

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource)]
struct DiagTimer(Timer);

#[derive(Resource)]
struct TrailTimer(Timer);

#[derive(Resource, Default)]
struct EnergyBaseline(Option<f32>);

#[derive(Resource)]
struct SimulationControls {
    paused: bool,
    speed_multiplier: f32,
    trail_lifetime: f32,
    gravity_multiplier: f32,
    coulomb_multiplier: f32,
    enable_coulomb: bool,
}

impl Default for SimulationControls {
    fn default() -> Self {
        Self {
            paused: false,
            speed_multiplier: 1.0,
            trail_lifetime: TRAIL_LIFETIME,
            gravity_multiplier: 1.0,
            coulomb_multiplier: 0.0, // Start disabled; users enable via slider
            enable_coulomb: true,
        }
    }
}

// ============================================================================
// Setup Systems
// ============================================================================

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let star_mass = 10_000.0; // Reduced from 200k - was causing 20x orbital velocity (instability)

    // Mini solar system — 6 planets with varied orbits
    //        name       dist   mass    radius  color
    let planets: &[(&str, f32, f32, f32, Color)] = &[
        ("Mercury", 80.0, 100.0, 4.0, ORANGE_RED.into()),
        ("Venus", 130.0, 400.0, 7.0, GOLD.into()),
        ("Earth", 190.0, 500.0, 8.0, DODGER_BLUE.into()),
        ("Mars", 260.0, 250.0, 5.0, INDIAN_RED.into()),
        ("Jupiter", 380.0, 2000.0, 16.0, BISQUE.into()),
        ("Neptune", 520.0, 800.0, 11.0, DEEP_SKY_BLUE.into()),
    ];

    let mut total_planet_momentum = Vec3::ZERO;
    let mut planet_data: Vec<(&str, Vec3, Vec3, f32, f32, Color, f32)> = Vec::new();

    info!("═══ System Setup ═══");

    // DEBUGGING: Open file to log all velocity calculations
    let mut debug_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("C:\\Users\\mathi\\AppData\\Local\\Temp\\nbody_setup_debug.txt")
        .ok();

    for (i, &(name, distance, mass, radius, color)) in planets.iter().enumerate() {
        let angle = (i as f32) * std::f32::consts::TAU / planets.len() as f32;

        let v = calculate_plummer_orbital_velocity(star_mass, distance, SOFTENING);
        let period = 2.0 * std::f32::consts::PI * distance / v;

        let pos = Vec3::new(distance * angle.cos(), distance * angle.sin(), 0.0);
        let vel = Vec3::new(-angle.sin() * v, angle.cos() * v, 0.0);
        let v_magnitude = vel.truncate().length();

        // Log to file
        if let Some(ref mut f) = debug_file {
            let _ = writeln!(
                f,
                "{}: r={:.1} v_calc={:.6} vel_vec={:.6} mass={:.0} momentum_contrib={:.6}",
                name,
                distance,
                v,
                v_magnitude,
                mass,
                mass * v_magnitude
            );
        }

        total_planet_momentum += mass * vel;

        info!("  {name:<8} r={distance:>3.0}  m={mass:>4.0}  v={v:>5.2}  T={period:>4.1}s");

        planet_data.push((name, pos, vel, mass, radius, color, period));
    }

    if let Some(mut f) = debug_file {
        let _ = writeln!(f, "\nTotal momentum: {:.6}", total_planet_momentum.length());
        let star_vel = -total_planet_momentum / star_mass;
        let _ = writeln!(f, "Star compensating vel: {:.6}", star_vel.length());
    }

    // Star gets compensating velocity for center-of-mass frame (p_total ≈ 0)
    let star_vel = -total_planet_momentum / star_mass;
    info!(
        "  Star     compensating v=({:.4}, {:.4}) for CoM frame",
        star_vel.x, star_vel.y
    );

    commands.spawn((
        Name::new("Star"),
        Sprite {
            color: YELLOW.into(),
            custom_size: Some(Vec2::splat(50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        Mass::new(star_mass),
        Velocity {
            linvel: star_vel,
            angvel: Vec3::ZERO,
        },
        AppliedForce::new(Vec3::ZERO),
        GravitySource,
        GravityAffected,
        CelestialBody { radius: 25.0 },
        Star,
    ));

    for (i, &(name, pos, vel, mass, radius, color, period)) in planet_data.iter().enumerate() {
        let v_mag = vel.length();
        let dist = pos.truncate().length();

        // Alternating charges: +/- pattern for electromagnetic interactions
        // Scaled small so gravity remains dominant but Coulomb is visible
        let charge_value = if i % 2 == 0 {
            -50.0 // Negative charge (Mercury, Earth, Jupiter)
        } else {
            50.0 // Positive charge (Venus, Mars, Neptune)
        };

        commands.spawn((
            Name::new(name.to_string()),
            Sprite {
                color,
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_translation(pos),
            Mass::new(mass),
            Velocity {
                linvel: vel,
                angvel: Vec3::ZERO,
            },
            AppliedForce::new(Vec3::ZERO),
            GravitySource,
            GravityAffected,
            Charge::new(charge_value),
            SofteningLength::default(),
            CelestialBody { radius },
            OrbitalRef {
                initial_dist: dist,
                initial_speed: v_mag,
                period,
            },
        ));
    }
}

fn spawn_starfield(mut commands: Commands) {
    // Procedural starfield — golden-angle spiral for uniform distribution
    let golden_angle = std::f32::consts::TAU * (1.0 - 1.0 / 1.618034);

    for i in 0..300u32 {
        let angle = i as f32 * golden_angle;
        // Hash-like distance variation
        let hash = ((i.wrapping_mul(2654435761)) % 1000) as f32 / 1000.0;
        let dist = 900.0 + hash * 1200.0;
        let size = 1.0 + (i % 3) as f32 * 0.5;
        let brightness = 0.2 + (i % 7) as f32 * 0.1;

        commands.spawn((
            Sprite {
                color: Color::srgba(brightness, brightness, brightness * 1.15, 0.7),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                dist * angle.cos(),
                dist * angle.sin(),
                -1.0, // Behind everything
            )),
        ));
    }
}

// Log actual velocity components right after setup (one frame later)
fn log_initial_velocities(bodies: Query<(&Name, &Transform, &Velocity), With<GravityAffected>>) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .open("C:\\Users\\mathi\\AppData\\Local\\Temp\\nbody_initial_velocities.txt")
    {
        let _ = writeln!(file, "═══ INITIAL VELOCITIES (after first frame) ═══");
        for (name, tf, vel) in &bodies {
            let r = tf.translation.truncate().length();
            let v = vel.linvel.length();
            let _ = writeln!(
                file,
                "{}: r={:.1} v={:.6} linvel=({:.4}, {:.4})",
                name, r, v, vel.linvel.x, vel.linvel.y
            );
        }
    }
}

// ============================================================================
// Visual Systems
// ============================================================================

fn spawn_trails(
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
    mut commands: Commands,
    controls: Res<SimulationControls>,
    planets: Query<(&Transform, &Sprite, &CelestialBody), (Without<Star>, With<GravityAffected>)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    for (transform, sprite, body) in &planets {
        let dot_size = (body.radius * 0.4).clamp(2.0, 5.0);

        commands.spawn((
            Sprite {
                color: sprite.color.with_alpha(0.5),
                custom_size: Some(Vec2::splat(dot_size)),
                ..default()
            },
            Transform::from_translation(transform.translation),
            TrailDot {
                lifetime: controls.trail_lifetime,
                max_lifetime: controls.trail_lifetime,
                color: sprite.color,
            },
        ));
    }
}

fn fade_trails(
    time: Res<Time>,
    mut commands: Commands,
    mut trails: Query<(Entity, &mut TrailDot, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut dot, mut sprite) in &mut trails {
        dot.lifetime -= dt;
        if dot.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Quadratic fade for smooth trail disappearance
        let t = dot.lifetime / dot.max_lifetime;
        let alpha = t * t * 0.5;
        sprite.color = dot.color.with_alpha(alpha);
    }
}

fn camera_follow(
    bodies: Query<&Transform, With<CelestialBody>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<CelestialBody>)>,
    time: Res<Time>,
) {
    let Ok(mut cam_tf) = camera.single_mut() else {
        return;
    };

    // Find max distance from origin for auto-zoom
    let mut max_dist: f32 = 0.0;
    for transform in &bodies {
        let pos = transform.translation.truncate();
        max_dist = max_dist.max(pos.length());
    }

    // Cap max_dist to prevent unbounded zoom-out (safety: 1200.0 is max useful view)
    max_dist = max_dist.min(1200.0).max(200.0);

    // Target scale: fit all bodies with 100px padding (viewport height = 720, half = 360px)
    let target_scale = (max_dist + 100.0) / 350.0;

    // Time-scale aware lerp: faster at high speeds, smoother transitions
    let dt = time.delta_secs();
    let lerp_speed = (0.05 * dt).min(0.5); // Clamp to avoid overshooting
    let current_scale = cam_tf.scale.x;
    let new_scale = current_scale.lerp(target_scale, lerp_speed);

    // Fixed camera at origin - no tracking, no drift!
    cam_tf.translation.x = 0.0;
    cam_tf.translation.y = 0.0;
    cam_tf.scale = Vec3::splat(new_scale);
}

// ============================================================================
// EGui Controls
// ============================================================================

fn egui_controls(mut contexts: EguiContexts, mut controls: ResMut<SimulationControls>) {
    // Using EguiPrimaryContextPass schedule ensures context is ready
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Simulation Controls")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("⏱ Physics Controls");

            // Speed multiplier slider
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(
                    egui::Slider::new(&mut controls.speed_multiplier, 0.1..=10.0)
                        .text("x")
                        .step_by(0.1),
                );
                if ui.button("Reset").clicked() {
                    controls.speed_multiplier = 1.0;
                }
            });

            if ui.checkbox(&mut controls.paused, "⏸ Paused").changed() {
                info!("Paused: {}", controls.paused);
            }

            ui.separator();

            ui.heading("🎨 Visual Settings");

            // Trail lifetime slider
            ui.horizontal(|ui| {
                ui.label("Trail Length:");
                ui.add(
                    egui::Slider::new(&mut controls.trail_lifetime, 0.5..=10.0)
                        .text("s")
                        .step_by(0.1),
                );
            });

            ui.separator();

            ui.heading("🌍 Physics Parameters");

            // Gravity multiplier slider
            ui.horizontal(|ui| {
                ui.label("Gravity:");
                ui.add(
                    egui::Slider::new(&mut controls.gravity_multiplier, 0.1..=5.0)
                        .text("x")
                        .step_by(0.1),
                );
            });

            ui.separator();

            ui.heading("⚡ Electromagnetic");

            if ui
                .checkbox(&mut controls.enable_coulomb, "Enable Coulomb Forces")
                .changed()
            {
                info!(
                    "Coulomb forces: {}",
                    if controls.enable_coulomb { "ON" } else { "OFF" }
                );
            }

            if controls.enable_coulomb {
                ui.horizontal(|ui| {
                    ui.label("Coulomb:");
                    ui.add(
                        egui::Slider::new(&mut controls.coulomb_multiplier, 0.0..=10.0)
                            .text("x")
                            .step_by(0.1),
                    );
                });
            }

            // Helpful info
            ui.separator();
            ui.small("💡 Adjust forces to see gravity-charge interactions");
        });
}

fn apply_controls(
    mut time: ResMut<Time<Virtual>>,
    controls: Res<SimulationControls>,
    mut gravity_params: ResMut<GravityParams>,
    mut coulomb_config: ResMut<CoulombConfig>,
) {
    // Pause/unpause simulation
    if controls.paused {
        time.pause();
    } else {
        time.unpause();
    }

    // Apply speed multiplier to time scaling
    time.set_relative_speed(controls.speed_multiplier);

    // Apply gravity multiplier (scales with user control)
    gravity_params.gravitational_constant =
        DEFAULT_GRAVITATIONAL_CONSTANT * controls.gravity_multiplier;

    // Apply Coulomb multiplier (scales with user control, respects enable_coulomb toggle)
    let coulomb_base = if controls.enable_coulomb { 1.0 } else { 0.0 };
    coulomb_config.coulomb_constant = 1.0 * controls.coulomb_multiplier * coulomb_base;
}

// ============================================================================
// Diagnostics
// ============================================================================

fn physics_diagnostics(
    time: Res<Time>,
    mut timer: ResMut<DiagTimer>,
    mut baseline: ResMut<EnergyBaseline>,
    bodies: Query<
        (&Name, &Transform, &Velocity, &Mass, Option<&OrbitalRef>),
        With<GravityAffected>,
    >,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut total_ke = 0.0;
    let mut total_pe = 0.0;
    let mut total_momentum = Vec3::ZERO;

    // Collect all bodies for PE calculation
    let body_data: Vec<_> = bodies
        .iter()
        .map(|(_, tf, vel, mass, _)| (tf.translation, vel.linvel, mass.value))
        .collect();

    // KE + momentum
    for &(_, linvel, mass) in &body_data {
        total_ke += 0.5 * mass * linvel.length_squared();
        total_momentum += mass * linvel;
    }

    // PE (pairwise, Plummer potential: Φ = -G·M/√(r²+ε²))
    for i in 0..body_data.len() {
        for j in (i + 1)..body_data.len() {
            let (pos_i, _, mass_i) = body_data[i];
            let (pos_j, _, mass_j) = body_data[j];
            let r = (pos_i - pos_j).length();
            let softened = (r * r + SOFTENING * SOFTENING).sqrt();
            total_pe -= DEFAULT_GRAVITATIONAL_CONSTANT * mass_i * mass_j / softened;
        }
    }

    let total_energy = total_ke + total_pe;

    // Track drift from initial energy
    let energy_drift = if let Some(e0) = baseline.0 {
        (total_energy - e0) / e0.abs() * 100.0
    } else {
        baseline.0 = Some(total_energy);
        0.0
    };

    let elapsed = time.elapsed_secs();
    let msg = format!(
        "═══ t={:.1}s ═══ KE={:.1} PE={:.1} E_drift={:+.4}% |p|={:.4}",
        elapsed,
        total_ke,
        total_pe,
        energy_drift,
        total_momentum.length()
    );

    info!("{}", msg);
    info!(
        "  Momentum |p|={:.4}  px={:.3} py={:.3}",
        total_momentum.length(),
        total_momentum.x,
        total_momentum.y
    );

    // Write to file for analysis
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("C:\\Users\\mathi\\AppData\\Local\\Temp\\nbody_diag.txt")
    {
        let _ = writeln!(file, "{}", msg);
        for (name, transform, velocity, mass, orbital_ref) in &bodies {
            let pos = transform.translation.truncate();
            let dist = pos.length();
            let speed = velocity.linvel.length();

            if let Some(orbital) = orbital_ref {
                let dist_drift = (dist - orbital.initial_dist) / orbital.initial_dist * 100.0;
                let speed_drift = (speed - orbital.initial_speed) / orbital.initial_speed * 100.0;
                let body_msg = format!(
                    "  {} r={:.1}({:+.1}%) v={:.2}({:+.1}%)",
                    name.as_str(),
                    dist,
                    dist_drift,
                    speed,
                    speed_drift
                );
                let _ = writeln!(file, "{}", body_msg);
                info!("{}", body_msg);
            } else {
                // Star
                let star_msg = format!(
                    "  {} pos=({:.1},{:.1}) v={:.2}",
                    name.as_str(),
                    pos.x,
                    pos.y,
                    speed
                );
                let _ = writeln!(file, "{}", star_msg);
                info!("{}", star_msg);
            }
        }
        let _ = writeln!(file);
    }
}
