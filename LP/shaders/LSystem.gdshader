shader_type canvas_item;

// Uniforms for configuration
uniform vec4 branch_color = vec4(0.0, 1.0, 0.0, 1.0);  // Default green color for branches
uniform vec4 root_color = vec4(0.4, 0.2, 0.0, 1.0);    // Default brown color for roots
uniform bool is_root = false;  // If true, use root color settings
uniform float depth_factor = 1.0;  // Factor to simulate depth variation
uniform float noise_strength = 0.05;  // Refined strength of color noise for more subtle natural variation
uniform float edge_highlight = 0.15;  // Slightly reduced strength of edge highlighting effect for smoother blending

// Simple noise function for natural color variation
float random(vec2 coords) {
    return fract(sin(dot(coords, vec2(12.9898, 78.233))) * 43758.5453);
}

void fragment() {
    // Choose color based on whether the instance is a root or a branch
    vec4 base_color = is_root ? root_color : branch_color;

    // Simulate depth-based color variation by modifying the base color's brightness
    float depth_modifier = 0.85 + 0.15 * depth_factor;  // Slightly refined depth modifier for more natural brightness variation
    vec4 modified_color = base_color * depth_modifier;
    
    // Add subtle noise variation to simulate natural imperfections
    float noise = random(FRAGCOORD.xy) * noise_strength;
    modified_color.rgb += noise;
    
    // Calculate edge highlighting effect based on fragment position
    float edge = abs(dFdx(FRAGCOORD.x)) + abs(dFdy(FRAGCOORD.y));
    edge = clamp(edge * edge_highlight, 0.0, 0.5);  // Reduce edge effect intensity for a softer look
    modified_color.rgb += vec3(edge);
    
    // Apply a subtle darkening to simulate ambient occlusion
    float ao_factor = clamp(1.0 - 0.2 * length(FRAGCOORD.xy) / 500.0, 0.7, 1.0);
    modified_color.rgb *= ao_factor;
    
    // Ensure alpha stays consistent
    modified_color.a = base_color.a;
    
    COLOR = modified_color;
}
