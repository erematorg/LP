use glam::{Vec2, Quat};
use std::collections::HashSet;

/// Represents the output of the interpreter: positions and directions for rendering
pub struct InterpreterOutput {
    pub positions: Vec<(Vec2, Vec2)>, // List of line segments (start, end)
}

/// Interprets L-System symbols and computes positions and directions
pub fn interpret(
    symbols: &str,
    rotation_angle: f32,
    line_length: f32,
) -> Result<InterpreterOutput, String> {
    let valid_symbols = HashSet::from(['F', '+', '-', '[', ']']);
    if symbols.chars().any(|ch| !valid_symbols.contains(&ch)) {
        return Err("Invalid symbol in L-System string".to_string());
    }

    let mut stack: Vec<(Vec2, Vec2, f32)> = Vec::new(); // Stack now stores position, direction, and current scale
    let mut position = Vec2::ZERO;
    let mut direction = Vec2::Y;
    let mut output = InterpreterOutput { positions: Vec::new() };

    let mut current_scale = 1.0; // Track consistent scaling per iteration

    for ch in symbols.chars() {
        match ch {
            'F' => {
                let scaled_length = line_length * current_scale; // Apply consistent scaling
                let new_position = position + direction * scaled_length;
                output.positions.push((position, new_position));
                position = new_position;
            }
            '+' => {
                direction = Quat::from_rotation_z(-rotation_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
            }
            '-' => {
                direction = Quat::from_rotation_z(rotation_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
            }
            '[' => {
                stack.push((position, direction, current_scale)); // Save scale state too
            }
            ']' => {
                if let Some((saved_position, saved_direction, saved_scale)) = stack.pop() {
                    position = saved_position;
                    direction = saved_direction;
                    current_scale = saved_scale; // Restore previous scale
                }
            }
            _ => {}
        }
    }

    Ok(output)
}
