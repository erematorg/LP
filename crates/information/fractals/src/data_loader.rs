use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    pub angle_range: [f32; 2],
    pub iterations_range: [usize; 2],
    pub scaling_factor_range: [f32; 2],
    pub segment_length_range: [f32; 2],
    pub curvature_factor_range: [f32; 2],
    pub depth_scale_factor_range: [f32; 2], // Controls scaling based on bracket depth
    pub angle_variation_range: [f32; 2], // Controls random variation in branch angles
    pub base_thickness_range: [f32; 2], // Controls base line thickness
    pub thickness_scale_factor_range: [f32; 2], // Controls how thickness reduces with depth
    pub directional_bias_range: [f32; 2], // Controls phototropism effect (upward growth bias)
    pub angle_evolution_range: [f32; 2], // Controls branch drooping effect over time
}

#[derive(Deserialize, Debug)]
pub struct Template {
    pub axiom: String,
    pub rules: std::collections::HashMap<String, String>,
    pub parameters: Parameters,
}

/// Load a template from the fractals.json file
pub fn load_template(template_name: &str) -> Result<Template, String> {
    let file_content = fs::read_to_string("crates/information/fractals/src/fractals.json")
        .map_err(|_| "Error: Could not read fractals.json".to_string())?;

    let json: serde_json::Value = serde_json::from_str(&file_content)
        .map_err(|_| "Error: Invalid JSON format in fractals.json".to_string())?;

    json["templates"].get(template_name)
        .ok_or_else(|| format!("Error: Template '{}' not found", template_name))
        .and_then(|template| serde_json::from_value(template.clone())
            .map_err(|_| format!("Error: Failed to parse template '{}'", template_name)))
}