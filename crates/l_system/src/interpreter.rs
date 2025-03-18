use bevy::prelude::*;
use std::collections::HashSet;

/// Represents the output of the interpreter: positions and directions for rendering
pub struct InterpreterOutput {
    pub positions: Vec<(Vec2, Vec2)>, // List of line segments (start, end)
    pub thicknesses: Vec<f32>,        // Thickness for each line segment
}

/// Interprets L-System symbols and computes positions and directions
pub fn interpret(
    symbols: &str,
    rotation_angle: f32,
    line_length: f32,
    scale_factor: f32,
    angle_variation: f32,
    base_thickness: f32,
    thickness_scale_factor: f32,
    directional_bias: f32,
    angle_evolution_factor: f32, // New parameter for angle evolution/drooping
) -> Result<InterpreterOutput, String> {
    let valid_symbols = HashSet::from(['F', '+', '-', '[', ']']);
    if symbols.chars().any(|ch| !valid_symbols.contains(&ch)) {
        return Err("Invalid symbol in L-System string".to_string());
    }

    // Structure to track branch information including age for angle evolution
    struct BranchState {
        position: Vec2,
        direction: Vec2,
        scale: f32,
        thickness: f32,
        segment_count: usize, // Track segment count for age-based angle evolution
    }

    let mut stack: Vec<BranchState> = Vec::new();
    let mut position = Vec2::ZERO;
    let mut direction = Vec2::Y;
    let mut output = InterpreterOutput { 
        positions: Vec::new(),
        thicknesses: Vec::new(),
    };

    let mut current_scale = 1.0;
    let mut bracket_depth = 0;
    let mut current_thickness = base_thickness;
    let mut segment_count = 0; // Track segments in current branch for age-based effects

    // Reference upward direction for phototropism
    let upward_direction = Vec2::Y;

    for ch in symbols.chars() {
        match ch {
            'F' => {
                // Apply directional bias toward upward direction (phototropism)
                if directional_bias > 0.0 {
                    // Calculate dot product to determine current alignment with upward direction
                    let alignment = direction.dot(upward_direction);
                    
                    // Only apply bias if not already pointing upward
                    if alignment < 0.99 {
                        // Calculate perpendicular direction toward upward
                        let perpendicular = (upward_direction - direction * alignment).normalize();
                        
                        // Apply gradual bias based on depth and bias strength
                        // Deeper branches have stronger phototropism
                        let bias_strength = directional_bias * (1.0 + 0.2 * bracket_depth as f32);
                        
                        // Blend current direction with upward bias
                        direction = (direction + perpendicular * bias_strength).normalize();
                    }
                }
                
                // Apply angle evolution (drooping effect) based on segment count
                if angle_evolution_factor > 0.0 {
                    // Calculate alignment with vertical axis (how vertical the branch is)
                    let vertical_alignment = direction.dot(upward_direction).abs();
                    
                    // Horizontal branches droop more than vertical ones
                    let horizontal_factor = 1.0 - vertical_alignment;
                    
                    // Age factor - older segments (higher segment count) droop more
                    let age_factor = (segment_count as f32) * 0.1;
                    
                    // Calculate drooping effect - stronger for horizontal branches and deeper nesting
                    let droop_strength = angle_evolution_factor * horizontal_factor * 
                                        (1.0 + age_factor) * (1.0 + 0.2 * bracket_depth as f32);
                    
                    // Apply subtle rotation toward downward direction
                    if droop_strength > 0.0 {
                        // Calculate direction to rotate (perpendicular to current, toward downward)
                        let droop_direction = Vec2::new(
                            -direction.y.signum() * droop_strength, 
                            direction.x.signum() * droop_strength
                        );
                        
                        // Apply drooping rotation
                        direction = (direction + droop_direction).normalize();
                    }
                }
                
                // Scale based on bracket depth
                let depth_scale = scale_factor.powf(bracket_depth as f32);
                let scaled_length = line_length * current_scale * depth_scale;
                
                // Calculate thickness based on depth
                let line_thickness = current_thickness * thickness_scale_factor.powf(bracket_depth as f32);
                
                let new_position = position + direction * scaled_length;
                output.positions.push((position, new_position));
                output.thicknesses.push(line_thickness);
                position = new_position;
                
                // Increment segment count for this branch
                segment_count += 1;
            }
            '+' => {
                // Apply rotation with angle variation based on bracket depth
                let variation_factor = angle_variation * bracket_depth as f32;
                let varied_angle = rotation_angle * (1.0 + variation_factor);
                
                direction = Quat::from_rotation_z(-varied_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
            }
            '-' => {
                // Apply rotation with angle variation based on bracket depth
                let variation_factor = angle_variation * bracket_depth as f32; 
                let varied_angle = rotation_angle * (1.0 + variation_factor);
                
                direction = Quat::from_rotation_z(varied_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
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
                segment_count = 0; // Reset segment count for new branch
            }
            ']' => {
                if let Some(state) = stack.pop() {
                    position = state.position;
                    direction = state.direction;
                    current_scale = state.scale;
                    current_thickness = state.thickness;
                    segment_count = state.segment_count; // Restore parent branch's segment count
                    bracket_depth -= 1;
                }
            }
            _ => {}
        }
    }

    Ok(output)
}