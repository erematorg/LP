use bevy::prelude::*;
use std::collections::HashSet;

/// Represents different types of L-System components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    /// S (Segment) - Basic building blocks like stems and branches
    Segment,
    /// B (Bifurcation) - Points where branching occurs
    Bifurcation,
    /// C (Core) - Origin/central elements like the trunk base
    Core,
    /// F (Legacy) - Original single-symbol used for all parts
    Legacy,
}

/// Represents the output of the interpreter: positions and directions for rendering
pub struct InterpreterOutput {
    pub positions: Vec<(Vec2, Vec2)>, // List of line segments (start, end)
    pub thicknesses: Vec<f32>,        // Thickness for each line segment
    pub types: Vec<SymbolType>,       // Type of each line segment
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
    angle_evolution_factor: f32,
) -> Result<InterpreterOutput, String> {
    // Update valid symbols to include both old and new symbol types
    let valid_symbols = HashSet::from(['F', 'S', 'B', 'C', '+', '-', '[', ']']);
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
        types: Vec::new(),
    };

    let mut current_scale = 1.0;
    let mut bracket_depth = 0;
    let mut current_thickness = base_thickness;
    let mut segment_count = 0; // Track segments in current branch for age-based effects

    // Reference upward direction for phototropism
    let upward_direction = Vec2::Y;

    for ch in symbols.chars() {
        match ch {
            // All drawing symbols (F, S, B, C) behave the same way for now
            // But we track their type for future differentiation
            'F' | 'S' | 'B' | 'C' => {
                // Determine symbol type
                let symbol_type = match ch {
                    'F' => SymbolType::Legacy,
                    'S' => SymbolType::Segment,
                    'B' => SymbolType::Bifurcation,
                    'C' => SymbolType::Core,
                    _ => unreachable!(), // Already filtered by valid_symbols
                };
                
                // Apply directional bias toward upward direction (phototropism)
                if directional_bias > 0.0 {
                    let alignment = direction.dot(upward_direction);
                    
                    if alignment < 0.99 {
                        let perpendicular = (upward_direction - direction * alignment).normalize();
                        let bias_strength = directional_bias * (1.0 + 0.2 * bracket_depth as f32);
                        direction = (direction + perpendicular * bias_strength).normalize();
                    }
                }
                
                // Apply angle evolution (drooping effect) based on segment count
                if angle_evolution_factor > 0.0 {
                    let vertical_alignment = direction.dot(upward_direction).abs();
                    let horizontal_factor = 1.0 - vertical_alignment;
                    let age_factor = (segment_count as f32) * 0.1;
                    
                    let droop_strength = angle_evolution_factor * horizontal_factor * 
                                        (1.0 + age_factor) * (1.0 + 0.2 * bracket_depth as f32);
                    
                    if droop_strength > 0.0 {
                        let droop_direction = Vec2::new(
                            -direction.y.signum() * droop_strength, 
                            direction.x.signum() * droop_strength
                        );
                        
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
                output.types.push(symbol_type); // Store the symbol type
                position = new_position;
                
                // Increment segment count for this branch
                segment_count += 1;
            },
            '+' => {
                // Apply rotation with angle variation based on bracket depth
                let variation_factor = angle_variation * bracket_depth as f32;
                let varied_angle = rotation_angle * (1.0 + variation_factor);
                
                direction = Quat::from_rotation_z(-varied_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
            },
            '-' => {
                // Apply rotation with angle variation based on bracket depth
                let variation_factor = angle_variation * bracket_depth as f32; 
                let varied_angle = rotation_angle * (1.0 + variation_factor);
                
                direction = Quat::from_rotation_z(varied_angle.to_radians())
                    .mul_vec3(direction.extend(0.0))
                    .truncate();
            },
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
            },
            ']' => {
                if let Some(state) = stack.pop() {
                    position = state.position;
                    direction = state.direction;
                    current_scale = state.scale;
                    current_thickness = state.thickness;
                    segment_count = state.segment_count; // Restore parent branch's segment count
                    bracket_depth -= 1;
                }
            },
            _ => {}
        }
    }

    Ok(output)
}