shader_type canvas_item;

uniform vec4 base_color = vec4(0.5, 0.5, 0.5, 1.0);  // Greyish color
uniform float edge_softness = 0.5;  // Controls the softness of the edges
uniform float highlight_intensity = 0.3;  // Intensity of the highlight effect

void fragment() {
    // Calculate the distance from the center of the particle
    vec2 uv = FRAGCOORD.xy / SCREEN_PIXEL_SIZE.xy;
    float dist = length(uv - vec2(0.5));

    // Set the alpha to be solid for now to ensure visibility
    float alpha = 1.0;

    // Add a very subtle highlight to the center of each particle for visibility
    float highlight = smoothstep(0.5, 0.0, dist) * highlight_intensity;

    // Set the color with the highlight added, maintaining solid alpha
    vec3 final_color = base_color.rgb + highlight;
    COLOR = vec4(final_color, alpha);
}
