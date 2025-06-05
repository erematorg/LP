use information::fractals::prelude::*;
use rand::prelude::*; //Will move to bevy_rand

fn main() {
    // Load the multi-symbol template from fractals.json
    let template = load_template("multi_tree").expect("Failed to load template");

    // Generate random parameters using rand 0.9
    let mut rng = rand::rng();

    let iterations = rng.random_range(
        template.parameters.iterations_range[0]..=template.parameters.iterations_range[1],
    );
    let angle =
        rng.random_range(template.parameters.angle_range[0]..=template.parameters.angle_range[1]);
    let scaling_factor = rng.random_range(
        template.parameters.scaling_factor_range[0]..=template.parameters.scaling_factor_range[1],
    );
    let segment_length = rng.random_range(
        template.parameters.segment_length_range[0]..=template.parameters.segment_length_range[1],
    );

    // Get depth scale factor for parametric growth
    let depth_scale_factor = rng.random_range(
        template.parameters.depth_scale_factor_range[0]
            ..=template.parameters.depth_scale_factor_range[1],
    );

    // Get angle variation factor for natural looking branches
    let angle_variation = rng.random_range(
        template.parameters.angle_variation_range[0]..=template.parameters.angle_variation_range[1],
    );

    // Get thickness parameters for realistic branch rendering
    let base_thickness = rng.random_range(
        template.parameters.base_thickness_range[0]..=template.parameters.base_thickness_range[1],
    );

    let thickness_scale_factor = rng.random_range(
        template.parameters.thickness_scale_factor_range[0]
            ..=template.parameters.thickness_scale_factor_range[1],
    );

    // Get directional bias factor for phototropism effect
    let directional_bias = rng.random_range(
        template.parameters.directional_bias_range[0]
            ..=template.parameters.directional_bias_range[1],
    );

    // Get angle evolution factor for branch drooping effect
    let angle_evolution_factor = rng.random_range(
        template.parameters.angle_evolution_range[0]..=template.parameters.angle_evolution_range[1],
    );

    // Create the L-System using the template
    let mut lsystem = LSystem::new(&template.axiom);
    for (symbol, replacement) in &template.rules {
        lsystem = lsystem.add_rule(symbol.chars().next().unwrap(), replacement);
    }
    lsystem = lsystem.set_iterations(iterations);

    // Generate the L-System output
    let output = lsystem.generate();

    // Print symbol usage statistics
    let mut symbols_count = std::collections::HashMap::new();
    for ch in output.chars() {
        if "SBCF+-[]".contains(ch) {
            *symbols_count.entry(ch).or_insert(0) += 1;
        }
    }

    println!("Symbol counts in generated system:");
    for (symbol, count) in symbols_count.iter() {
        let description = match symbol {
            'S' => "Segment - Basic building block",
            'B' => "Bifurcation - Splitting point",
            'C' => "Core - Origin/central element",
            'F' => "Legacy - Original symbol",
            '+' => "Rotate clockwise",
            '-' => "Rotate counter-clockwise",
            '[' => "Push state",
            ']' => "Pop state",
            _ => "Unknown",
        };
        println!("{}: {} ({})", symbol, count, description);
    }

    // Render the L-System using randomized values
    run_renderer(
        &output,
        angle,
        scaling_factor,
        segment_length,
        depth_scale_factor,
        angle_variation,
        base_thickness,
        thickness_scale_factor,
        directional_bias,
        angle_evolution_factor,
    );
}
