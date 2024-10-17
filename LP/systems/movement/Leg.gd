@tool
extends LimbBase
class_name Leg

# Parameters specific to leg functionality
@export var walk_speed: float = 10.0
@export var step_height: float = 5.0
@export var is_grounded: bool = false
@export var target_forward_max_reach: float = 50.0
@export var target_backwards_max_reach: float = 30.0
@export var leg_maximum_height: float = 20.0

# Onready references for ground detection (e.g., RayCast2D)
#@onready var ground_raycast: RayCast2D = $GroundRaycast
@onready var ground_cast : ShapeCast2D = $GroundCast

# Signals for ground interaction
signal touched_ground(leg: Leg)
signal left_ground(leg: Leg)

# Ready function for initialization
func _ready():
	initialize_components()
	# Ensure ground detection is properly set up
	ensure_ground_raycast()

# Ensures that the RayCast2D for ground detection exists
func ensure_ground_raycast():
	if !ground_cast:
		ground_cast = ShapeCast2D.new()
		ground_cast.name = "GroundCast"
		add_child(ground_cast, true)
		ground_cast.set_owner(get_tree().get_edited_scene_root())
		ground_cast.target_position = (Vector2)(0,80)
		ground_cast.shape = CapsuleShape2D.new()
		ground_cast.max_results = 6

# Function to handle walking logic
func walk(direction: Vector2):
	if is_grounded:
		position += direction * walk_speed * get_process_delta_time()

# Update function for checking ground status
func _physics_process(_delta):
	#ground_raycast.force_raycast_update()
	ground_cast.force_shapecast_update()
	check_ground_status()
	adjust_leg_target()

# Checks if the leg is touching the ground
func check_ground_status():
	if ground_cast.is_colliding():
		if !is_grounded:
			is_grounded = true
			emit_signal("touched_ground", self)
	else:
		if is_grounded:
			is_grounded = false
			emit_signal("left_ground", self)

# Adjusts the target marker position to simulate stepping movement
func adjust_leg_target():
	if !is_grounded:
		pass
		#var new_target_position = global_position + Vector2.RIGHT * target_forward_max_reach
		#new_target_position.y = leg_maximum_height
		#target_marker.global_position = new_target_position
	elif is_grounded:
		target_marker.global_position = ground_cast.get_collision_point(0)
		#target_marker.global_position = global_position + Vector2.RIGHT * step_height
