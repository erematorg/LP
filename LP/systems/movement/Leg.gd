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
#@onready var ground_cast : ShapeCast2D = $GroundCast
@export_category("Rays")
var ground_rays : Array[RayCast2D] 
var first_ground_hit : Vector2
@export var ray_count : int = 6 :
	set(val):
		ray_count = clampi(val, 2, 30)
		call_deferred("ensure_ground_raycast")
@export var max_angle: float = 180 :
	set(val):
		max_angle = val
		call_deferred("ensure_ground_raycast")
@export var radius : int = 160 :
	set(val):
			radius = val
			call_deferred("ensure_ground_raycast")
@export var base_rotation : float = PI / 2 : # Rotation offset in radians, 0 = right, PI/2 = up, PI = left
	set(val):
		base_rotation = val
		call_deferred("ensure_ground_raycast")
@export var spread_factor = 0.5 :  # Adjust this between 0 (tight) and 1 (full spread)
	set(val):
		spread_factor = val
		call_deferred("ensure_ground_raycast")
		
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
	while ground_rays.size() > ray_count and ground_rays.size() > 0:
		var ray_to_remove = ground_rays.pop_back()  # Remove the last ray from the array
		ray_to_remove.queue_free()  # Free it from the scene
	
	ground_rays.resize(ray_count)
	for i in ground_rays.size():
		if !ground_rays[i]:
			ground_rays[i] = RayCast2D.new()
			ground_rays[i].name = "GroundRay_"+str(i)
			add_child(ground_rays[i])
		
		var start_angle = -max_angle / 2 * spread_factor
		var end_angle = max_angle / 2 * spread_factor
		var angle = lerp(start_angle, end_angle, float(i) / (ray_count - 1))
		# Add base rotation to control the hemisphere's orientation
		var final_angle = angle + base_rotation
		# Convert the angle to a direction vector (polar to cartesian conversion)
		var direction = Vector2(cos(final_angle), sin(final_angle))
		# Set the target position using the calculated direction and ray length (radius)
		ground_rays[i].target_position = direction * radius


# Function to handle walking logic
func walk(direction: Vector2):
	if is_grounded:
		position += direction * walk_speed * get_process_delta_time()


# Update function for checking ground status
func _physics_process(_delta):
	for i in ground_rays.size():
		ground_rays[i].force_raycast_update()
	check_ground_status()
	adjust_leg_target()


# Checks if the leg is touching the ground
func check_ground_status():
	var closest_distance = INF
	var closest_ground_hit = null
	var leg_position = global_position  # The position of the leg or the base point for raycasting

	# Iterate over all the rays and find the closest collision point
	for i in range(ground_rays.size()):
		if ground_rays[i].is_colliding():
			var hit_position = ground_rays[i].get_collision_point()
			var distance = target_marker.position.distance_to(hit_position) #leg_position.distance_to(hit_position)

			# If this hit is closer than previous hits, store it
			if distance < closest_distance:
				closest_distance = distance
				closest_ground_hit = hit_position

	# If we found a ground hit, update the grounded status and move leg IK to the closest hit
	if closest_ground_hit != null:
		if !is_grounded:
			is_grounded = true
			emit_signal("touched_ground", self)

		first_ground_hit = closest_ground_hit  # Move the leg IK to this closest hit
	else:
		# If no ground is touched, update grounded status
		if is_grounded:
			is_grounded = false
			emit_signal("left_ground", self)

	#for i in ground_rays.size():
		#if ground_rays[i].is_colliding():
			##if !is_grounded:
			#is_grounded = true
			#emit_signal("touched_ground", self)
			#first_ground_hit = ground_rays[i].get_collision_point()
			#return
#
	#if is_grounded:
		#is_grounded = false
		#emit_signal("left_ground", self)
		
	#Old old code
	#if ground_cast.is_colliding():
	#	if !is_grounded:
	#		is_grounded = true
	#		emit_signal("touched_ground", self)
	#else:
	#	if is_grounded:
	#		is_grounded = false
	#		emit_signal("left_ground", self)


# Adjusts the target marker position to simulate stepping movement
func adjust_leg_target():
	if !is_grounded:
		pass
		#var new_target_position = global_position + Vector2.RIGHT * target_forward_max_reach
		#new_target_position.y = leg_maximum_height
		#target_marker.global_position = new_target_position
	elif is_grounded and first_ground_hit != Vector2.ZERO:
		target_marker.global_position = first_ground_hit
		#target_marker.global_position = global_position + Vector2.RIGHT * step_height
