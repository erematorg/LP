use glam::{Vec2, Quat};
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
    directional_bias: f32, // New parameter for directional growth bias
) -> Result<InterpreterOutput, String> {
    let valid_symbols = HashSet::from(['F', '+', '-', '[', ']']);
    if symbols.chars().any(|ch| !valid_symbols.contains(&ch)) {
        return Err("Invalid symbol in L-System string".to_string());
    }

    let mut stack: Vec<(Vec2, Vec2, f32, f32)> = Vec::new(); // Position, direction, scale, thickness
    let mut position = Vec2::ZERO;
    let mut direction = Vec2::Y;
    let mut output = InterpreterOutput { 
        positions: Vec::new(),
        thicknesses: Vec::new(),
    };

    let mut current_scale = 1.0; // Track consistent scaling per iteration
    let mut bracket_depth = 0; // Track current bracket nesting depth
    let mut current_thickness = base_thickness; // Start with base thickness

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
                
                // Scale based on bracket depth
                let depth_scale = scale_factor.powf(bracket_depth as f32);
                let scaled_length = line_length * current_scale * depth_scale;
                
                // Calculate thickness based on depth
                let line_thickness = current_thickness * thickness_scale_factor.powf(bracket_depth as f32);
                
                let new_position = position + direction * scaled_length;
                output.positions.push((position, new_position));
                output.thicknesses.push(line_thickness);
                position = new_position;
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
                stack.push((position, direction, current_scale, current_thickness));
                bracket_depth += 1; // Increase depth when entering a branch
            }
            ']' => {
                if let Some((saved_position, saved_direction, saved_scale, saved_thickness)) = stack.pop() {
                    position = saved_position;
                    direction = saved_direction;
                    current_scale = saved_scale;
                    current_thickness = saved_thickness;
                    bracket_depth -= 1; // Decrease depth when leaving a branch
                }
            }
            _ => {}
        }
    }

    Ok(output)
}