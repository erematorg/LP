extends Node2D

class_name LSystemRenderer

var l_system: LSystem
var offset: Vector2 = Vector2(500, 350)  # Offset to ensure visibility
var current_stage: Dictionary
var default_angle: float
var default_length: float

# Connect to the signal from LSystemManager to get updates
func _ready():
	# Get the LSystemManager node and connect to its signal
	var manager = get_parent().get_node("LSystemManager")
	if manager != null:
		manager.connect("l_system_changed", Callable(self, "_on_l_system_changed"))
	set_process(true)  # Enable _process callback

# Update the L-System instance and redraw
func _on_l_system_changed(new_l_system: LSystem):
	l_system = new_l_system
	# Save default values from the L-System for consistency
	default_angle = l_system.angle
	default_length = l_system.length
	queue_redraw()  # Trigger a redraw

# Process callback to update the wind effect based on the cursor position
func _process(delta):
	queue_redraw()

# Render the L-System string with wind effect
func _draw() -> void:
	if l_system == null:
		return

	var l_string = l_system.generate()
	var pos = offset
	var angle = -PI / 2  # Start facing upwards
	var stack = []

	var mouse_pos = get_global_mouse_position()
	var wind_radius = 100.0  # Radius within which the wind affects the segments
	var wind_strength = 0.05  # Base wind effect strength

	for c in l_string:
		match c:
			"F":
				var new_pos = pos + Vector2(cos(angle), sin(angle)) * default_length
				var distance_to_mouse = pos.distance_to(mouse_pos)
				
				if distance_to_mouse < wind_radius:
					var wind_angle = (mouse_pos - pos).angle()
					var wind_effect = wind_strength * (wind_radius - distance_to_mouse) / wind_radius
					angle += wind_effect * sin(wind_angle - angle)
				
				# Use current_stage color if available
				draw_line(pos, new_pos, current_stage.color if current_stage else Color(1, 1, 1))
				pos = new_pos
			"F*":
				var new_pos_star = pos + Vector2(cos(angle), sin(angle)) * default_length
				draw_line(pos, new_pos_star, current_stage.color if current_stage else Color(1, 1, 1))
				pos = new_pos_star
				draw_circle(new_pos_star, 5, current_stage.fruit_color if "fruit_color" in current_stage else Color(1, 0, 0))
			"+":
				angle += deg_to_rad(default_angle)
			"-":
				angle -= deg_to_rad(default_angle)
			"[":
				stack.append([pos, angle])
			"]":
				var state = stack.pop_back()
				pos = state[0]
				angle = state[1]
				draw_ellipse(pos, Vector2(10, 5), current_stage.color if current_stage else Color(0.13, 0.55, 0.13))

# Draw an ellipse (utility function)
func draw_ellipse(center: Vector2, size: Vector2, color: Color):
	draw_rect(Rect2(center - size / 2, size), color)

# Method to handle stage change
func change_stage(stage_index: int):
	var manager = get_parent().get_node("LSystemManager")
	if manager != null:
		# Ensure the stage index is valid
		if stage_index >= 0 and stage_index < manager.lifecycle_stages.size():
			current_stage = manager.lifecycle_stages[stage_index]
			queue_redraw()
		else:
			print("Invalid stage index:", stage_index)
