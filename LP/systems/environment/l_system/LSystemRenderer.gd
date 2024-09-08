extends Node2D

class_name LSystemRenderer

var l_system: LSystem
var offset: Vector2 = Vector2(500, 350)  # Offset to ensure visibility
var current_stage: Dictionary
var default_angle: float
var default_length: float

# Store cached random values for angles and lengths
var random_angles: Array = []
var random_lengths: Array = []

# Variables for stage colors and fruit color
var current_stage_color: Color = Color(1, 1, 1)
var fruit_color: Color = Color(1, 0, 0)  # Red fruit as default

# Connect to the signal from LSystemManager to get updates
func _ready():
	var manager = get_parent().get_node("LSystemManager")
	if manager != null:
		if manager.connect("l_system_changed", Callable(self, "_on_l_system_changed")) == OK:
			print("Connected to l_system_changed signal")
		else:
			print("Failed to connect to l_system_changed signal")
	set_process(true)  # Enable _process callback

# Update the L-System instance and redraw
func _on_l_system_changed(new_l_system: LSystem):
	l_system = new_l_system
	# Save default values from the L-System for consistency
	default_angle = l_system.angle
	default_length = l_system.length
	_generate_random_values()  # Generate random values once
	queue_redraw()  # Trigger a redraw

# Generate random values for angles and lengths (once per L-System update)
func _generate_random_values():
	random_angles.clear()
	random_lengths.clear()
	
	var l_string = l_system.generate()
	for c in l_string:
		if c == "F" or c == "F*":
			# Store random values for each segment
			random_angles.append(deg_to_rad(default_angle + randf_range(-5, 5)))
			random_lengths.append(default_length * randf_range(0.9, 1.1))

# Process callback to update the wind effect based on the cursor position
func _process(delta):
	queue_redraw()

# Helper function to calculate wind effect based on mouse position
func calculate_wind_effect(pos: Vector2, angle: float, wind_strength: float, wind_radius: float, mouse_pos: Vector2) -> float:
	var distance_to_mouse = pos.distance_to(mouse_pos)
	if distance_to_mouse < wind_radius:
		var wind_angle = (mouse_pos - pos).angle()
		var wind_effect = wind_strength * (wind_radius - distance_to_mouse) / wind_radius
		angle += wind_effect * sin(wind_angle - angle)
	return angle

# Render the L-System string with wind effect and cached randomness
func _draw() -> void:
	if l_system == null:
		return

	var l_string = l_system.generate()
	var pos = offset
	var angle = -PI / 2  # Start facing upwards
	var stack = []
	var mouse_pos = get_global_mouse_position()
	var wind_radius = 100.0  # Wind affect radius
	var wind_strength = 0.05  # Wind effect strength
	var random_index = 0  # Index for accessing random values

	for c in l_string:
		match c:
			"F":
				# Use cached random length and angle for natural growth
				var rand_length = random_lengths[random_index]
				angle = calculate_wind_effect(pos, angle, wind_strength, wind_radius, mouse_pos)

				var new_pos = pos + Vector2(cos(angle), sin(angle)) * rand_length

				# Use current_stage color if available
				draw_line(pos, new_pos, current_stage_color)
				pos = new_pos
				random_index += 1  # Move to the next cached random value
			"F*":
				var rand_length_star = random_lengths[random_index]
				var new_pos_star = pos + Vector2(cos(angle), sin(angle)) * rand_length_star
				draw_line(pos, new_pos_star, current_stage_color)
				pos = new_pos_star
				draw_circle(new_pos_star, 5, fruit_color)
				random_index += 1
			"+":
				# Use cached random angle for branches
				angle += random_angles[random_index]
			"-":
				angle -= random_angles[random_index]
			"[":
				stack.append([pos, angle])
			"]":
				var state = stack.pop_back()
				pos = state[0]
				angle = state[1]
				draw_ellipse(pos, Vector2(10, 5), current_stage_color)

# Draw an ellipse (utility function)
func draw_ellipse(center: Vector2, size: Vector2, color: Color):
	draw_rect(Rect2(center - size / 2, size), color)

# Method to handle lifecycle stage change
func change_stage(stage_index: int):
	var manager = get_parent().get_node("LSystemManager")
	if is_instance_valid(manager) and stage_index >= 0 and stage_index < manager.lifecycle_stages.size():
		current_stage = manager.lifecycle_stages[stage_index]

		# Update stage color and fruit color based on lifecycle stage
		current_stage_color = current_stage.color
		if current_stage.has("fruit_color"):
			fruit_color = current_stage.fruit_color
		
		queue_redraw()
	else:
		print("Invalid stage index or manager is invalid")

# Clean-up function to prevent memory leaks
func _exit_tree():
	l_system = null
