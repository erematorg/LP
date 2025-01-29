use glam::{Vec2, Quat};
use std::collections::HashSet;

/// Represents the output of the interpreter: positions and directions for rendering
pub struct InterpreterOutput {
    pub positions: Vec<(Vec2, Vec2)>, // List of line segments (start, end)
}

/// Interprets L-System symbols and computes positions and directions
pub fn interpret(symbols: &str, rotation_angle: f32, line_length: f32) -> Result<InterpreterOutput, String> {
    let valid_symbols = HashSet::from(['F', '+', '-', '[', ']']);
    if symbols.chars().any(|ch| !valid_symbols.contains(&ch)) {
        return Err("Invalid symbol in L-System string".to_string());
    }

    let mut stack: Vec<(Vec2, Vec2)> = Vec::new();
    let mut position = Vec2::ZERO;
    let mut direction = Vec2::Y;
    let mut output = InterpreterOutput { positions: Vec::new() };

    for ch in symbols.chars() {
        match ch {
            'F' => {
                let new_position = position + direction * line_length;
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
            '[' => stack.push((position, direction)),
            ']' => if let Some((saved_position, saved_direction)) = stack.pop() {
                position = saved_position;
                direction = saved_direction;
            },
            _ => {}
        }
    }

    Ok(output)
}
