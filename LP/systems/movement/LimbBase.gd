@tool
extends Path2D
class_name LimbBase

@export var update : bool = false

# Exported variables for control in the editor
@export_range(0.1, 90) var render_resolution: float = 100 :
	set(val):
		render_resolution = val
		call_deferred("update_visual")

@export var line_thickness: float = 10.0 :
	set(val):
		line_thickness = val
		call_deferred("update_visual")

@export var enable_ik: bool = false :
	set(val):
		enable_ik = val
		call_deferred("update_visual")

@export var width_curve: Curve = Curve.new() :
	set(val):
		width_curve = val
		call_deferred("update_visual")

@export var joint_mobility_limit: Vector2 = Vector2(0.2, 0.2) :
	set(val):
		joint_mobility_limit = val
		call_deferred("update_visual")

@export var joint_constraints: Array[Vector2] = [] :
	set(val):
		joint_constraints = val
		call_deferred("update_visual")

@export var manual_points: bool = false :
	set(val):
		manual_points = val
		call_deferred("update_visual")

@export var fabrik_chain_length: int = 5 :
	set(val):
		fabrik_chain_length = val
		call_deferred("update_visual")

@export var ik_target_position: Vector2 = Vector2.ZERO :
	set(val):
		ik_target_position = val
		call_deferred("update_visual")

@export var damping: float = 0.1 :
	set(val):
		damping = val
		call_deferred("update_visual")

@export var stiffness: float = 0.5 :
	set(val):
		stiffness = val
		call_deferred("update_visual")

# Onready references for scene nodes
@onready var skeleton: Skeleton2D = $Skeleton2D
@onready var target_marker: Marker2D = $target
@onready var line_renderer: Line2D = $Line2D

# Variables for rendering and joints
var visual_points: PackedVector2Array = PackedVector2Array()
var bones: Array = []
var deformed_curve: Curve2D = Curve2D.new()

# --- Ready and Initialization ---
#func _ready():
	#initialize_components()
#Commented because this should be done in inheriting class

# Initialize essential components such as the skeleton, Line2D, target marker, and visual updates
func initialize_components():
	ensure_width_curve()
	ensure_line_renderer()
	ensure_skeleton()
	ensure_skeleton_joint_angle()
	ensure_target_marker()
	
	# Connect signals for visual updates
	curve.connect("changed", update_visual)
	connect("property_list_changed", update_visual)
	
	update_visual()

# --- Ensuring Nodes ---
func ensure_width_curve():
	# Ensure the width curve has at least one point
	if width_curve.point_count < 1:
		width_curve.add_point(Vector2(0, 1))

func ensure_line_renderer():
	# Create and configure Line2D if it doesn't exist
	if !line_renderer:
		line_renderer = Line2D.new()
		configure_line_renderer(line_renderer)
		add_child(line_renderer, true)
		line_renderer.set_owner(get_tree().get_edited_scene_root())

func configure_line_renderer(renderer: Line2D):
	# Set rendering styles for the line renderer
	renderer.end_cap_mode = Line2D.LINE_CAP_ROUND
	renderer.begin_cap_mode = Line2D.LINE_CAP_ROUND
	renderer.joint_mode = Line2D.LINE_JOINT_ROUND
	renderer.antialiased = true  # Enable antialiasing for smoother lines

func ensure_skeleton():
	# Create a Skeleton2D node if it doesn't exist
	if !skeleton:
		skeleton = Skeleton2D.new()
		add_child(skeleton, true)
		skeleton.set_owner(get_tree().get_edited_scene_root())
		
func ensure_skeleton_joint_angle():
	pass
	#var skeleton_mod = skeleton.get_modification_stack()
	#var CCDIK : SkeletonModification2DCCDIK = skeleton_mod.get_modification(0)
	#CCDIK.ccdik_data_chain_length = skeleton.get_bone_count()
	#print("nr of bones: " + str(skeleton.get_bone_count()))
	#print("nr of chains in CCDIK: " + str(CCDIK.ccdik_data_chain_length))
	#for i in CCDIK.ccdik_data_chain_length:
	#	CCDIK.set_ccdik_joint_enable_constraint(i, true)
	#	CCDIK.set_ccdik_joint_constraint_angle_min(i,-60.0)
	#	CCDIK.set_ccdik_joint_constraint_angle_max(i, 60.0)

func ensure_target_marker():
	# Create a target marker if it doesn't exist and set its position
	if !target_marker:
		target_marker = Marker2D.new()
		target_marker.name = "target"
		target_marker.position = curve.get_point_position(curve.point_count - 1)
		add_child(target_marker, true)
		target_marker.set_owner(get_tree().get_edited_scene_root())

# --- Visual Updates ---
func update_visual():
	# Generate the rig and offset the curve to the origin
	_generate_rig()
	#offset_curve_to_origin()
	
	# Set visual points based on manual points or tessellate the curve
	if manual_points:
		visual_points.resize(curve.point_count)
		for i in range(curve.point_count):
			visual_points[i] = curve.get_point_position(i)
	else:
		visual_points = curve.tessellate(5, render_resolution)
	
	render_curve()

func offset_curve_to_origin():
	# Offset the curve points to start from the origin
	if curve.get_point_position(0) != Vector2.ZERO:
		var offset = curve.get_point_position(0)
		for i in range(curve.point_count):
			curve.set_point_position(i, curve.get_point_position(i) - offset)

# --- Skeleton and IK ---
func _generate_rig():
	# Clear existing bones and create new ones along the curve
	clear_existing_bones()
	create_bones_along_curve()
	
	# Configure IK if enabled
	if enable_ik:
		var points: PackedVector2Array = get_curve_points()
		_configure_ik(points)

func get_curve_points() -> PackedVector2Array:
	# Retrieve the points along the curve, considering manual points or tessellation
	# Get points along the curve based on manual points or tessellation
	var curve_points: PackedVector2Array
	if manual_points:
		curve_points.resize(curve.point_count)
		for i in range(curve.point_count):
			curve_points[i] = curve.get_point_position(i)
	else:
		curve_points = curve.tessellate(5, render_resolution)
	return curve_points

func clear_existing_bones():
	# Free all existing bone nodes in the skeleton
	for bone in skeleton.get_children():
		bone.queue_free()
	bones.clear()

# Create bones along the curve
func create_bones_along_curve():
	# Preparing to create bones by calculating curve points
	# Create bones along the curve points
	var curve_points: PackedVector2Array = get_curve_points()
	
	# Resize joint constraints and set defaults
	joint_constraints.resize(curve_points.size() - 1)
	set_default_joint_constraints()
	
	curve_points = offset_curve_points(curve_points)
	bones.resize(curve_points.size())

	var current_bone: Node2D = skeleton
	for i in range(curve_points.size() - 1):
		current_bone = create_bone(current_bone, curve_points, i)
		bones[i] = current_bone

	# Add a tip at the last point
	var tip_node = Node2D.new()
	bones[curve_points.size() - 1] = tip_node
	current_bone.add_child(tip_node, true)
	tip_node.global_position = curve_points[curve_points.size() - 1]
	tip_node.set_owner(get_tree().get_edited_scene_root())

func set_default_joint_constraints():
	# Set default joint constraints based on the mobility limit
	for i in range(joint_constraints.size()):
		if joint_constraints[i] == null:
			joint_constraints[i] = joint_mobility_limit

func offset_curve_points(points: PackedVector2Array) -> PackedVector2Array:
	# Offset all curve points by the current position
	for i in range(points.size()):
		points[i] += position
	return points

func create_bone(current_bone: Node2D, curve_points: PackedVector2Array, i: int) -> Bone2D:
	# Create a new bone for the given segment of the curve, setting up length and angle
	# Create a bone node and set its properties
	var new_bone: Bone2D = Bone2D.new()
	new_bone.set_autocalculate_length_and_angle(false)
	current_bone.add_child(new_bone, true)
	new_bone.global_position = curve_points[i]
	new_bone.rest = Transform2D(0.0, new_bone.position)
	new_bone.set_length(new_bone.global_position.distance_to(curve_points[i + 1]))
	new_bone.set_bone_angle(new_bone.global_position.angle_to_point(curve_points[i + 1]))
	new_bone.set_owner(get_tree().get_edited_scene_root())
	return new_bone

func _configure_ik(curve_points: PackedVector2Array):
	# Configure the IK solver using the specified curve points
	var modification_stack = SkeletonModificationStack2D.new()
	var ik_solver = SkeletonModification2DCCDIK.new()
	configure_ik_solver(ik_solver, curve_points)
	modification_stack.add_modification(ik_solver)
	modification_stack.enabled = enable_ik
	skeleton.set_modification_stack(modification_stack)

func configure_ik_solver(ik_solver: SkeletonModification2DCCDIK, curve_points: PackedVector2Array):
	# Set parameters for the IK solver
	ik_solver.ccdik_data_chain_length = min(curve_points.size() - 1, fabrik_chain_length)
	ik_solver.target_nodepath = target_marker.get_path()

	for i in range(curve_points.size() - 1):
		if i >= fabrik_chain_length:
			break
		ik_solver.set_ccdik_joint_enable_constraint(i, true)
		ik_solver.set_ccdik_joint_constraint_angle_invert(i, true)
		ik_solver.set_ccdik_joint_constraint_angle_max(i, joint_constraints[i].x)
		ik_solver.set_ccdik_joint_constraint_angle_min(i, -joint_constraints[i].y)
		ik_solver.set_ccdik_joint_bone_index(i, i)
		ik_solver.set_ccdik_joint_bone2d_node(i, bones[i].get_path())

	ik_solver.tip_nodepath = bones[bones.size() - 1].get_path()

# --- Rendering ---
func _process(_delta):
	# Update curve rendering each frame
	if(update == true):
		render_curve()

func render_curve():
	# Render the curve based on bone positions
	deformed_curve.clear_points()
	visual_points.resize(bones.size())
	
	for i in range(bones.size()):
		visual_points[i] = to_local(bones[i].global_position)
		deformed_curve.add_point(to_local(bones[i].global_position))
	
	line_renderer.points = visual_points
	line_renderer.width_curve = width_curve
	line_renderer.width = line_thickness
	ensure_skeleton_joint_angle()
