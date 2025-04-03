use bevy::prelude::*;
use std::collections::HashSet;

/// Represents different types of L-System components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum SymbolType {
    Segment,     // S - Basic building blocks like stems and branches
    Bifurcation, // B - Points where branching occurs
    Core,        // C - Origin/central elements like trunk base
    Legacy,      // F - Original single-symbol used for all parts
}

/// Represents the output of the interpreter: positions and directions for rendering
pub struct InterpreterOutput {
    pub positions: Vec<(Vec2, Vec2)>, // List of line segments (start, end)
    pub thicknesses: Vec<f32>,        // Thickness for each line segment
    pub types: Vec<SymbolType>,       // Type of each line segment
}

/// Interprets L-System symbols and computes positions and directions for rendering
pub fn interpret(
    symbols: &str,
    rotation_angle: f32,
    line_length: f32,
    scale_factor: f32,
    angle_variation: f32,
    base_thickness: f32,
    thickness_scale_factor: f32,
    directional_bias: f32,
    angle_evolution_factor: f32,
) -> Result<InterpreterOutput, String> {
    // Validate input symbols
    let valid_symbols = HashSet::from(['F', 'S', 'B', 'C', '+', '-', '[', ']']);
    if symbols.chars().any(|ch| !valid_symbols.contains(&ch)) {
        return Err("Invalid symbol in L-System string".to_string());
    }

    // Branch state
    struct BranchState {
        position: Vec2,
        direction: Vec2,
        scale: f32,
        thickness: f32,
        segment_count: usize,
    }

    let mut stack: Vec<BranchState> = Vec::new();
    let mut position = Vec2::ZERO;
    let mut direction = Vec2::Y;
    let mut output = InterpreterOutput {
        positions: Vec::new(),
        thicknesses: Vec::new(),
        types: Vec::new(),
    };
    let mut current_scale = 1.0;
    let mut bracket_depth = 0;
    let mut current_thickness = base_thickness;
    let mut segment_count = 0;
    let upward_direction = Vec2::Y;

    for ch in symbols.chars() {
        match ch {
            'F' | 'S' | 'B' | 'C' => {
                // Determine symbol type
                let symbol_type = match ch {
                    'F' => SymbolType::Legacy,
                    'S' => SymbolType::Segment,
                    'B' => SymbolType::Bifurcation,
                    'C' => SymbolType::Core,
                    _ => unreachable!(),
                };

                // Apply directional bias (phototropism)
                if directional_bias > 0.0 && direction.dot(upward_direction) < 0.99 {
                    let perpendicular = (upward_direction
                        - direction * direction.dot(upward_direction))
                    .normalize();
                    let bias_strength = directional_bias * (1.0 + 0.2 * bracket_depth as f32);
                    direction = (direction + perpendicular * bias_strength).normalize();
                }

                // Apply angle evolution (drooping)
                if angle_evolution_factor > 0.0 {
                    let vertical_alignment = direction.dot(upward_direction).abs();
                    let horizontal_factor = 1.0 - vertical_alignment;
                    let age_factor = segment_count as f32 * 0.1;
                    let droop_strength = angle_evolution_factor
                        * horizontal_factor
                        * (1.0 + age_factor)
                        * (1.0 + 0.2 * bracket_depth as f32);

                    if droop_strength > 0.0 {
                        let droop_direction = Vec2::new(
                            -direction.y.signum() * droop_strength,
                            direction.x.signum() * droop_strength,
                        );
                        direction = (direction + droop_direction).normalize();
                    }
                }

                // Create segment
                let depth_scale = scale_factor.powf(bracket_depth as f32);
                let scaled_length = line_length * current_scale * depth_scale;
                let line_thickness =
                    current_thickness * thickness_scale_factor.powf(bracket_depth as f32);
                let new_position = position + direction * scaled_length;

                output.positions.push((position, new_position));
                output.thicknesses.push(line_thickness);
                output.types.push(symbol_type);
                position = new_position;
                segment_count += 1;
            }
            '+' | '-' => {
                // Handle rotation
                let variation_factor = angle_variation * bracket_depth as f32;
                let varied_angle = rotation_angle * (1.0 + variation_factor);
                let sign = if ch == '+' { -1.0 } else { 1.0 };

                direction = Quat::from_rotation_z(sign * varied_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate()
                    // Enhanced rotation with natural curvature and subtle directional bias later will inherit from forces such as gravity from the forces crate
                    .rotate_towards(upward_direction, angle_variation * 0.1)
                    .lerp(upward_direction, directional_bias * 0.05);
            }
            '[' => {
                stack.push(BranchState {
                    position,
                    direction,
                    scale: current_scale,
                    thickness: current_thickness,
                    segment_count,
                });
                bracket_depth += 1;
                segment_count = 0;
            }
            ']' => {
                if let Some(state) = stack.pop() {
                    position = state.position;
                    direction = state.direction;
                    current_scale = state.scale;
                    current_thickness = state.thickness;
                    segment_count = state.segment_count;
                    bracket_depth -= 1;
                }
            }
            _ => {}
        }
    }

    Ok(output)
}
