use l_system::{lsystem::LSystem, data_loader::load_template, renderer};
use rand::{rng, Rng}; // ✅ FIXED: Correct imports for `rand 0.9`

fn main() {
    // Load the branching_tree template from fractals.json
    let template = load_template("branching_tree").expect("Failed to load template");

    // Generate random parameters using `rand 0.9`
    let mut rng = rng(); 
    
    let iterations = rng.random_range(template.parameters.iterations_range[0]..=template.parameters.iterations_range[1]); 
    let angle = rng.random_range(template.parameters.angle_range[0]..=template.parameters.angle_range[1]); // ✅ FIXED
    let scaling_factor = rng.random_range(template.parameters.scaling_factor_range[0]..=template.parameters.scaling_factor_range[1]); 
    let segment_length = rng.random_range(template.parameters.segment_length_range[0]..=template.parameters.segment_length_range[1]); 
    
    // Get depth scale factor for parametric growth
    let depth_scale_factor = rng.random_range(
        template.parameters.depth_scale_factor_range[0]..=template.parameters.depth_scale_factor_range[1]
    );
    
    // Get angle variation factor for natural looking branches
    let angle_variation = rng.random_range(
        template.parameters.angle_variation_range[0]..=template.parameters.angle_variation_range[1]
    );
    
    // Get thickness parameters for realistic branch rendering
    let base_thickness = rng.random_range(
        template.parameters.base_thickness_range[0]..=template.parameters.base_thickness_range[1]
    );
    
    let thickness_scale_factor = rng.random_range(
        template.parameters.thickness_scale_factor_range[0]..=template.parameters.thickness_scale_factor_range[1]
    );

    // Create the L-System using the template
    let mut lsystem = LSystem::new(&template.axiom);
    for (symbol, replacement) in &template.rules {
        lsystem = lsystem.add_rule(symbol.chars().next().unwrap(), replacement);
    }
    lsystem = lsystem.set_iterations(iterations);

    // Generate the L-System output
    let output = lsystem.generate();
    println!("Generated Output:\n{}", output);

    // Render the L-System using randomized values
    renderer::run_renderer(
        &output, 
        angle, 
        scaling_factor, 
        segment_length, 
        depth_scale_factor, 
        angle_variation,
        base_thickness,
        thickness_scale_factor
    );
}