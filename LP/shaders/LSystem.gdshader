shader_type canvas_item;

uniform vec4 base_color = vec4(1.0, 0.65, 0.0, 1.0);  // Solid orange

void fragment() {
    // Apply a solid color (orange) for testing
    COLOR = base_color;
}
