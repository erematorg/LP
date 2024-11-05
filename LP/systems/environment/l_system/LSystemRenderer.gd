extends Node2D

class_name LSystemRenderer

# L-System variables
var l_system: LSystem
var root_system: LSystem
var offset: Vector2 = Vector2(500, 350)
var current_stage: Dictionary
var default_angle: float
var default_length: float

# Rendering configuration
@export var use_multimesh: bool = true  # Enable MultiMesh by default
var branch_count: int
var leaf_count: int

# MultiMesh setup
var multimesh_instance: MultiMeshInstance2D
var branch_multimesh: MultiMesh
var root_multimesh_instance: MultiMeshInstance2D
var root_multimesh: MultiMesh
var leaf_multimesh_instance: MultiMeshInstance2D
var leaf_multimesh: MultiMesh

# Branch density control
@export var branch_density: float = 1.0
@export var root_density: float = 1.5  # Increased root density for more realistic appearance

# Tapering and Bezier curve parameters
@export var base_thickness: float = 10.0
@export var tip_thickness: float = 2.0
@export var taper_factor: float = 0.85  # Enhanced smoother tapering for a more natural look
@export var bezier_strength_base: float = 0.3
@export var bezier_strength_tip: float = 0.1

# Color configuration (for shader usage)
@export var branch_color: Color = Color(0.0, 1.0, 0.0)
@export var root_color: Color = Color(0.4, 0.2, 0.0)
@export var leaf_color: Color = Color(0.0, 0.8, 0.0)

# Wind effect parameters
@export var wind_radius: float = 100.0
@export var wind_strength: float = 0.05  # Base wind strength
@export var test_wind: bool = false  # Enable/disable dynamic wind testing
@export var wind_speed: float = 1.0  # Speed of wind strength oscillation
@export var max_wind_strength: float = 0.3  # Increased maximum wind strength during oscillation for enhanced wind effect

# Cache for L-System string
var cos_angle_cache: Dictionary = {}
var sin_angle_cache: Dictionary = {}
var cached_l_string: String
var cached_root_string: String
var time_passed: float = 0.0  # Time tracker for wind oscillation

# Cached mesh sizes
var cached_branch_mesh_size: Vector2
var cached_root_mesh_size: Vector2
var cached_leaf_mesh_size: Vector2

# Called when the node enters the scene tree
func _ready():
	_connect_to_l_system_manager()

	if use_multimesh:
		_prepare_multimesh()
		_prepare_root_multimesh()
		_prepare_leaf_multimesh()
		apply_shader()  # Apply the shader

	set_process(true)

# Connect to LSystemManager signal
func _connect_to_l_system_manager():
	var manager = get_parent().get_node("LSystemManager")
	if manager != null:
		manager.connect("l_system_changed", Callable(self, "_on_l_system_changed"))
		manager.connect("lifecycle_stage_changed", Callable(self, "_on_lifecycle_stage_changed"))

# Prepare the MultiMesh for branches
func _prepare_multimesh():
	multimesh_instance = MultiMeshInstance2D.new()
	branch_multimesh = MultiMesh.new()
	branch_multimesh.transform_format = MultiMesh.TRANSFORM_2D

	# Basic quad mesh for each branch
	var branch_mesh := QuadMesh.new()
	branch_mesh.size = Vector2(12, 2.5)  # Slightly increased size for enhanced branch visibility
	branch_multimesh.mesh = branch_mesh
	cached_branch_mesh_size = branch_mesh.size
	
	multimesh_instance.multimesh = branch_multimesh
	add_child(multimesh_instance)

# Prepare the MultiMesh for roots
func _prepare_root_multimesh():
	root_multimesh_instance = MultiMeshInstance2D.new()
	root_multimesh = MultiMesh.new()
	root_multimesh.transform_format = MultiMesh.TRANSFORM_2D

	# Basic quad mesh for each root
	var root_branch_mesh := QuadMesh.new()
	root_branch_mesh.size = Vector2(10, 2)  # Adjust size to match root thickness and length
	root_multimesh.mesh = root_branch_mesh
	cached_root_mesh_size = root_branch_mesh.size
	
	root_multimesh_instance.multimesh = root_multimesh
	add_child(root_multimesh_instance)

# Prepare the MultiMesh for leaves
func _prepare_leaf_multimesh():
	leaf_multimesh_instance = MultiMeshInstance2D.new()
	leaf_multimesh = MultiMesh.new()
	leaf_multimesh.transform_format = MultiMesh.TRANSFORM_2D

	# Basic quad mesh for each leaf
	var leaf_mesh := QuadMesh.new()
	leaf_mesh.size = Vector2(5, 5)  # Adjust size for leaf representation
	leaf_multimesh.mesh = leaf_mesh
	cached_leaf_mesh_size = leaf_mesh.size
	
	leaf_multimesh_instance.multimesh = leaf_multimesh
	add_child(leaf_multimesh_instance)

# Apply the shader to the MultiMeshInstance2D
func apply_shader():
	# Load the shader from the .gdshader file
	var shader = load("res://shaders/LSystem.gdshader") as Shader

	# Branch material
	var branch_material = ShaderMaterial.new()
	branch_material.shader = shader
	branch_material.set_shader_parameter("is_root", false)  # Set shader parameter directly
	branch_material.set_shader_parameter("branch_color", branch_color)  # Set branch color from inspector
	multimesh_instance.material = branch_material

	# Root material
	var root_material = ShaderMaterial.new()
	root_material.shader = shader
	root_material.set_shader_parameter("is_root", true)  # Set shader parameter directly
	root_material.set_shader_parameter("root_color", root_color)  # Set root color from inspector
	root_multimesh_instance.material = root_material

	# Leaf material
	var leaf_material = ShaderMaterial.new()
	leaf_material.shader = shader
	leaf_material.set_shader_parameter("is_leaf", true)  # Set shader parameter directly
	leaf_material.set_shader_parameter("leaf_color", leaf_color)  # Set leaf color from inspector
	leaf_multimesh_instance.material = leaf_material

	# Debugging to confirm shader application
	print("Branch Shader Material Assigned: ", multimesh_instance.material)
	print("Root Shader Material Assigned: ", root_multimesh_instance.material)
	print("Leaf Shader Material Assigned: ", leaf_multimesh_instance.material)

# Handle L-System updates
func _on_l_system_changed(new_l_system: LSystem, new_root_system: LSystem):
	if new_l_system != null:
		l_system = new_l_system
	if new_root_system != null:
		root_system = new_root_system

	if l_system == null or root_system == null:
		push_error("L-System or Root-System is null. Skipping update.")
		return

	var new_l_string = l_system.generate()
	print("Generated L-System String: ", new_l_string)  # Print L-System string to verify leaf generation
	if new_l_string != cached_l_string:
		cached_l_string = new_l_string
		branch_count = int(cached_l_string.length() * branch_density)  # Adjust branch count based on density
		branch_multimesh.instance_count = branch_count
	
	# Count the number of leaves
	leaf_count = cached_l_string.count("L")
	leaf_multimesh.instance_count = leaf_count

	var new_root_string = root_system.generate()
	if new_root_string != cached_root_string:
		cached_root_string = new_root_string
		var root_count = int(cached_root_string.length() * root_density)
		root_multimesh.instance_count = root_count
	
	if root_multimesh.instance_count > 0:
		_update_multimesh()

# Handle lifecycle stage updates
func _on_lifecycle_stage_changed(stage_index: int, stage_data: Dictionary):
	if stage_data.has("color"):
		branch_color = stage_data["color"]
		root_color = stage_data["color"]
		leaf_color = stage_data["color"]
		apply_shader()
		print("Lifecycle stage changed. Updated colors to: ", branch_color)

# Generate and position branches using MultiMesh, applying wind effect and Bezier-like curvature
func _generate_multimesh_branches():
	var pos = offset
	var angle = -PI / 2
	var stack = []
	var index = 0
	var thickness = base_thickness
	var mouse_pos = get_global_mouse_position()
	var depth = 0

	for symbol in cached_l_string:
		match symbol:
			"F":
				# Calculate cosine and sine of angle once and cache it
				_cache_angle_values(angle)
				var cos_angle = cos_angle_cache[angle]
				var sin_angle = sin_angle_cache[angle]
				
				# Calculate next position with dynamic length based on depth
				var length = l_system.length * lerp(1.0, 0.5, float(depth) / branch_count)
				var next_pos = pos + Vector2(cos_angle, sin_angle) * length

				# Apply wind effect based on mouse position and dynamic wind strength
				var distance_to_mouse = pos.distance_to(mouse_pos)
				var dynamic_wind_strength = wind_strength

				if test_wind:
					# Oscillate wind strength over time for testing purposes
					dynamic_wind_strength += sin(time_passed * wind_speed) * max_wind_strength
				
				if distance_to_mouse < wind_radius:
					var wind_angle = (mouse_pos - pos).angle()
					var wind_effect = dynamic_wind_strength * (wind_radius - distance_to_mouse) / wind_radius
					angle += wind_effect * sin(wind_angle - angle)

				# Calculate cubic Bezier curve for more precise curvature
				var bezier_strength = lerp(bezier_strength_base, bezier_strength_tip, float(depth) / branch_count)
				var control1 = pos + Vector2(cos_angle, sin_angle) * (length * bezier_strength)
				var control2 = next_pos - Vector2(cos_angle, sin_angle) * (length * bezier_strength)
				var control3 = next_pos

				# Calculate cubic Bezier mid-point
				var t = 0.5
				var bezier_pos = (1 - t) * (1 - t) * (1 - t) * pos + \
					3 * (1 - t) * (1 - t) * t * control1 + \
					3 * (1 - t) * t * t * control2 + \
					t * t * t * control3
				
				# Cache transformation
				var branch_transform = Transform2D().rotated(angle)
				branch_transform.origin = bezier_pos
				
				# Set the branch in MultiMesh
				_set_multimesh_branch(index, branch_transform, thickness)
				
				# Adjust thickness based on depth for more visual variety
				thickness = max(base_thickness * pow(taper_factor, depth), tip_thickness)

				pos = next_pos
				depth += 1
				index += 1
			"L":
				# Handle leaf positioning
				var leaf_transform = Transform2D().translated(pos)
				_set_multimesh_leaf(index, leaf_transform)
				index += 1
			"+": angle += deg_to_rad(l_system.angle)
			"-": angle -= deg_to_rad(l_system.angle)
			"[": stack.append({"pos": pos, "angle": angle, "thickness": thickness, "depth": depth})
			"]":
				var state = stack.pop_back()
				pos = state["pos"]
				angle = state["angle"]
				thickness = state["thickness"]
				depth = state["depth"]

# Generate and position roots using MultiMesh, applying wind effect
func _generate_multimesh_roots():
	var pos = offset + Vector2(0, l_system.length * 0.5)  # Offset roots slightly below the branches to ensure linkage
	var angle = PI / 2
	var stack = []
	var index = 0
	var thickness = base_thickness * 0.7  # Adjusted root thickness for more natural tapering
	var mouse_pos = get_global_mouse_position()
	var depth = 0

	for symbol in cached_root_string:
		match symbol:
			"R":
				# Calculate cosine and sine of angle once and cache it
				_cache_angle_values(angle)
				var cos_angle = cos_angle_cache[angle]
				var sin_angle = sin_angle_cache[angle]
				
				# Calculate next position for root with dynamic length based on depth
				var length = root_system.length * lerp(1.0, 0.5, float(depth) / branch_count)
				var next_pos = pos + Vector2(cos_angle, sin_angle) * length

				# Apply wind effect based on mouse position and dynamic wind strength
				var distance_to_mouse = pos.distance_to(mouse_pos)
				var dynamic_wind_strength = wind_strength

				if test_wind:
					# Oscillate wind strength over time for testing purposes
					dynamic_wind_strength += sin(time_passed * wind_speed) * max_wind_strength
				
				if distance_to_mouse < wind_radius:
					var wind_angle = (mouse_pos - pos).angle()
					var wind_effect = dynamic_wind_strength * (wind_radius - distance_to_mouse) / wind_radius
					angle += wind_effect * sin(wind_angle - angle)

				# Cache transformation
				var root_transform = Transform2D().rotated(angle)
				root_transform.origin = pos

				# Set the root in MultiMesh
				_set_multimesh_root(index, root_transform, thickness)
				
				# Adjust thickness based on depth for more visual variety
				thickness = max(base_thickness * 0.7 * pow(taper_factor, depth), tip_thickness * 0.7)

				pos = next_pos
				depth += 1
				index += 1
			"+": angle += deg_to_rad(root_system.angle)
			"-": angle -= deg_to_rad(root_system.angle)
			"[": stack.append({"pos": pos, "angle": angle, "thickness": thickness, "depth": depth})
			"]":
				var state = stack.pop_back()
				pos = state["pos"]
				angle = state["angle"]
				thickness = state["thickness"]
				depth = state["depth"]

# Set the transform for each branch in the MultiMesh
func _set_multimesh_branch(index: int, branch_transform: Transform2D, thickness: float):
	# Adjust the size of the branch based on thickness
	var new_size = Vector2(l_system.length, thickness)
	if !is_equal_approx(cached_branch_mesh_size.x, new_size.x) or !is_equal_approx(cached_branch_mesh_size.y, new_size.y):
		cached_branch_mesh_size = new_size
		branch_multimesh.mesh.size = cached_branch_mesh_size

	branch_multimesh.set_instance_transform_2d(index, branch_transform)

# Set the transform for each root in the MultiMesh
func _set_multimesh_root(index: int, root_transform: Transform2D, thickness: float):
	# Adjust the size of the root based on thickness
	var new_size = Vector2(root_system.length, thickness)
	if cached_root_mesh_size != new_size:
		cached_root_mesh_size = new_size
		root_multimesh.mesh.size = cached_root_mesh_size
	root_multimesh_instance.multimesh.set_instance_transform_2d(index, root_transform)

# Set the transform for each leaf in the MultiMesh
func _set_multimesh_leaf(index: int, leaf_transform: Transform2D):
	leaf_multimesh.set_instance_transform_2d(index, leaf_transform)

# Update process to handle dynamic wind and redraw branches
func _process(delta):
	if test_wind:
		time_passed += delta  # Track time for dynamic wind strength oscillation
	_update_multimesh()

# Unified function to update both branches and roots
func _update_multimesh():
	_generate_multimesh_branches()
	_generate_multimesh_roots()

# Handle lifecycle stage transitions
#TODO: Still not fully implemented yet
func change_stage(stage_index: int):
	var manager = get_parent().get_node("LSystemManager")
	if manager != null and stage_index >= 0 and stage_index < manager.lifecycle_stages.size():
		current_stage = manager.lifecycle_stages[stage_index]
		
		# Update appearance, like color, size, etc., if needed
		var new_l_string = l_system.generate()
		if new_l_string != cached_l_string:
			cached_l_string = new_l_string
		
		var new_root_string = root_system.generate()
		if new_root_string != cached_root_string:
			cached_root_string = new_root_string
		
		_update_multimesh()
		queue_redraw()
	else:
		print("Invalid stage index:", stage_index)

# Cache angle values to reduce redundant calculations
func _cache_angle_values(angle: float):
	if not angle in cos_angle_cache:
		cos_angle_cache[angle] = cos(angle)
		sin_angle_cache[angle] = sin(angle)
