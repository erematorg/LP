extends Node2D

class_name LSystemRenderer

# L-System variables
var l_system: LSystem
var offset: Vector2 = Vector2(500, 350)
var current_stage: Dictionary
var default_angle: float
var default_length: float

# Wind effect parameters
@export var wind_radius: float = 100.0
@export var wind_strength: float = 0.05

# Rendering configuration
@export var use_multimesh: bool = true  # Enable MultiMesh by default
var branch_count: int

# MultiMesh setup
var multimesh_instance: MultiMeshInstance2D
var branch_multimesh: MultiMesh

# Tapering and Bezier curve parameters
@export var base_thickness: float = 10.0   # Balanced base thickness
@export var tip_thickness: float = 2.0     # Thicker tips for better balance
@export var taper_factor: float = 0.85     # Slower tapering for a smoother transition
@export var bezier_strength_base: float = 0.3   # Control for how strong the curve is at the base
@export var bezier_strength_tip: float = 0.1    # Control for how strong the curve is at the tip

# Branch density control
@export var branch_density: float = 1.0   # Controls how dense the branches are (scales number of branches)

#TODO
# Color configuration (for testing purposes)
@export var base_color: Color = Color(1.0, 0.65, 0.0)  # Orange for base
@export var branch_color: Color = Color(0.0, 1.0, 0.0)  # Green for branches

# Called when the node enters the scene tree
func _ready():
	_connect_to_l_system_manager()

	if use_multimesh:
		_prepare_multimesh()

	set_process(true)

# Connect to LSystemManager signal
func _connect_to_l_system_manager():
	var manager = get_parent().get_node("LSystemManager")
	if manager != null:
		manager.connect("l_system_changed", Callable(self, "_on_l_system_changed"))

# Prepare the MultiMesh for branches
func _prepare_multimesh():
	multimesh_instance = MultiMeshInstance2D.new()
	branch_multimesh = MultiMesh.new()
	branch_multimesh.transform_format = MultiMesh.TRANSFORM_2D

	# Basic quad mesh for each branch
	var branch_mesh := QuadMesh.new()
	branch_mesh.size = Vector2(10, 2) # Adjust size to match branch thickness and length
	branch_multimesh.mesh = branch_mesh
	
	multimesh_instance.multimesh = branch_multimesh
	add_child(multimesh_instance)

# Handle L-System updates
func _on_l_system_changed(new_l_system: LSystem):
	l_system = new_l_system
	branch_count = int(l_system.generate().length() * branch_density)  # Adjust branch count based on density
	branch_multimesh.instance_count = branch_count
	_generate_multimesh_branches()

# Generate and position branches using MultiMesh, applying wind effect and Bezier-like curvature
func _generate_multimesh_branches():
	var l_string = l_system.generate()
	var pos = offset
	var angle = -PI / 2
	var stack = []
	var index = 0  # Track which branch instance we're setting
	var thickness = base_thickness  # Start with base thickness
	var mouse_pos = get_global_mouse_position()
	var depth = 0

	for symbol in l_string:
		match symbol:
			"F":
				# Calculate next position
				var next_pos = pos + Vector2(cos(angle), sin(angle)) * l_system.length
				
				# Apply wind effect based on mouse position
				var distance_to_mouse = pos.distance_to(mouse_pos)
				if distance_to_mouse < wind_radius:
					var wind_angle = (mouse_pos - pos).angle()
					var wind_effect = wind_strength * (wind_radius - distance_to_mouse) / wind_radius
					angle += wind_effect * sin(wind_angle - angle)

				# Calculate Bezier-like transformations by adjusting segment positions
				var bezier_strength = lerp(bezier_strength_base, bezier_strength_tip, float(depth) / branch_count)
				var control1 = pos + Vector2(cos(angle), sin(angle)) * (l_system.length * bezier_strength)
				var control2 = next_pos - Vector2(cos(angle), sin(angle)) * (l_system.length * bezier_strength)

				# Adjust position using control points to simulate curvature
				var mid_point = (control1 + control2) * 0.5
				
				# Determine color based on depth (base or branch)
				var current_color = base_color if depth == 0 else branch_color
				
				# Set the branch in MultiMesh with the correct color
				_set_multimesh_branch(index, mid_point, angle, thickness, current_color)
				
				# Update thickness for tapering
				thickness *= taper_factor  
				thickness = max(thickness, tip_thickness)  

				pos = next_pos
				depth += 1
				index += 1
			"+": angle += deg_to_rad(l_system.angle)
			"-": angle -= deg_to_rad(l_system.angle)
			"[": stack.append([pos, angle, thickness, depth])
			"]":
				var state = stack.pop_back()
				pos = state[0]
				angle = state[1]
				thickness = state[2]
				depth = state[3]  # Restore thickness and depth for branch continuation

# Set the transform for each branch in the MultiMesh
func _set_multimesh_branch(index: int, start_pos: Vector2, angle: float, thickness: float, color: Color):
	var transform = Transform2D().rotated(angle)
	transform.origin = start_pos
	
	# Adjust the size of the branch based on thickness
	branch_multimesh.mesh.size = Vector2(l_system.length, thickness)  # Apply tapering

	#TODO Apply color
	branch_multimesh.set_instance_transform_2d(index, transform)
	# Set color per instance once supported
	# branch_multimesh.set_instance_color(index, color)

# Update process to handle dynamic wind and redraw branches
func _process(delta):
	_generate_multimesh_branches()  # Continuously reapply branch transformations

# Handle lifecycle stage transitions
func change_stage(stage_index: int):
	var manager = get_parent().get_node("LSystemManager")
	if manager != null and stage_index >= 0 and stage_index < manager.lifecycle_stages.size():
		current_stage = manager.lifecycle_stages[stage_index]
		
		# Update appearance, like color, size, etc., if needed
		_generate_multimesh_branches()
		queue_redraw()
	else:
		print("Invalid stage index:", stage_index)
