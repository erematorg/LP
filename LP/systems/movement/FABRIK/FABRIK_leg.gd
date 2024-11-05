class_name FABRIKLeg
extends Skeleton2D

## Emits a signal when the leg touches the ground.
signal touched_ground(leg: FABRIKLeg)
## Emits a signal when the leg leaves the ground.
signal left_ground(leg: FABRIKLeg)

@onready var ground_raycast: RayCast2D = %GroundRaycast
@onready var target: Marker2D = %Target

# -------- LOGIC COMPONENT --------

## Determines how the leg should move.
@export var leg_logic: FABRIKLegLogic

## This will cause the logic component to overwrite the stat values of the leg.
@export var use_logic_component_stat_values := true

# -------- COLLISION/MOVEMENT --------

## The maximum distance the leg can reach when moving along the defined direction.
@export var target_forward_max_reach := 0.0

## The maximum distance the leg can reach when moving opposite to the defined direction.
@export var target_backwards_max_reach := 0.0

## The maximum height the leg can reach.
@export var leg_maximum_height := 0.0

## True if the leg is touching the ground.
var is_on_floor := false

## The direction in which the creature is facing. Used for animation and collision detection.
var direction: Vector2 = Vector2.RIGHT

# -------- VISUAL --------

## Renders the leg behind the parent.
@export var background_leg := false

## The speed at which the leg repositions itself to the target. Useful for representing emotions (ex: fear, anger, calm, etc.)
@export var limb_reposition_speed := 0.0

## The default position of the target. Will be used when the leg is resting.
var default_target_position := Vector2.RIGHT * 2

# -------- FABRIK --------
## The textures to use for the leg. The textures should be limbs with the origin at the center top, and standing straight up.
@export var FABRIK_limbs_info: Array[FABRIKLegInfo]

# FABRIK settings
var FABRIK_modification: SkeletonModification2DFABRIK


#-------- MISC --------
## The limbs of the leg.
var limbs: Array[Bone2D] = []

## True if the leg has been generated.
var generated := false


func _ready():
	set_background_mode(background_leg)


func _process(_delta):
	if !generated: return

	check_floor_collision()

	if is_instance_valid(leg_logic):
		leg_logic.leg_callback(self)
	else:
		printerr("Leg doesn't have a logic component attached.")
		queue_free()
	
# -------- COLLISION --------

## Called when the leg touches the floor.
func on_floor_collision() -> void:
	# Touches the ground
	attempt_movement(global_position + Vector2.RIGHT * 10)


## Checks if the leg is touching the floor and handles collision.
func check_floor_collision() -> void:
	if ground_raycast.is_colliding():
		# Emit signal
		if !is_on_floor:
			touched_ground.emit(self)
			on_floor_collision()
		
		is_on_floor = true
	else:
		if is_on_floor:
			left_ground.emit(self)
			on_leg_left_floor()
			
		is_on_floor = false

## Runs when the leg leaves the floor.
func on_leg_left_floor() -> void:
	rest()

# -------- LEG GENERATION --------

## Generates the limbs from the given limb info array.
func generate(limb_info_array: Array[FABRIKLegInfo]) -> void:
	if generated: return
		
	# Error checking
	if !is_instance_valid(leg_logic):
		printerr("Leg doesn't have a logic component attached.")
		return
	
	if limb_info_array.is_empty():
		printerr("Leg doesn't have any limbs.")
		return
	
	# Setup
	setup_limbs(limb_info_array)
	setup_FABRIK_modification_stack(limb_info_array)
	
	generated = true

## Generates the limbs from the given limb info array. 
func setup_limbs(limb_info_array: Array[FABRIKLegInfo]) -> void:
	# Creates children
	var last_parent = self
	
	for info in limb_info_array:
		var texture := info.texture

		var limb := generate_limb(texture)
		
		last_parent.add_child(limb)
		last_parent = limb

		limbs.append(limb)

## Sets up the modification stack.
func setup_FABRIK_modification_stack(limb_info_array: Array[FABRIKLegInfo]) -> void:
	# Modification stack setup
	var modification_stack = SkeletonModificationStack2D.new()

	# Configure FABRIK modifier
	FABRIK_modification = SkeletonModification2DFABRIK.new()
	FABRIK_modification.fabrik_data_chain_length = limbs.size()
	FABRIK_modification.target_nodepath = target.get_path()
	
	# Configures every joint in the FABRIK chain
	for index in limbs.size():
		FABRIK_modification.set_fabrik_joint_bone_index(index, limbs[index].get_index())
		FABRIK_modification.set_fabrik_joint_bone2d_node(index, limbs[index].get_path())
		FABRIK_modification.set_fabrik_joint_magnet_position(index, limb_info_array[index].magnet)
		limbs[index].set_meta("default_magnet", limb_info_array[index].magnet)
	
	modification_stack.add_modification(FABRIK_modification)

	set_modification_stack(modification_stack)

	modification_stack.enabled = true


## Generates a limb from a texture. The texture should be a limb with the origin at the center top, and standing straight up.
func generate_limb(texture: Texture2D) -> Bone2D:
	# Bone creation
	var new_bone := Bone2D.new()
	new_bone.set_autocalculate_length_and_angle(false)
	new_bone.set_length(texture.get_height())
	
	# Sprite creation
	var new_sprite := Sprite2D.new()
	new_bone.set_meta("sprite", new_sprite)

	new_sprite.texture = texture

	new_bone.add_child(new_sprite)
	new_sprite.position = Vector2.ZERO
	new_sprite.position = Vector2(texture.get_height() / 2.0, 0)
	new_sprite.rotation_degrees = 90

	return new_bone

# -------- LEG CONTROL --------

## Returns true if the target is out of range.
func is_target_out_of_range() -> bool:
	var dist: float = target.global_position.distance_to(global_position)
	
	if !is_moving_in_direction(direction):
		# If the creature is moving in the same direction of the leg, the leg can reach further
		return dist > target_forward_max_reach * 1.1
	else:
		# If the creature is moving in the opposite direction of the leg, the leg can reach less
		return dist > target_backwards_max_reach

## Returns true if the creature is moving in the same direction as the one provided.
func is_moving_in_direction(compare_direction: Vector2) -> bool:
	return horizontal_velocity_direction() == sign(compare_direction.x)

## Returns the direction of the horizontal velocity. Ex: 1 if moving right, -1 if moving left.
func horizontal_velocity_direction() -> int:
	return sign(global_position.angle_to(target.global_position))

## Returns the progress of the leg's movement. Returns true if successful, false otherwise.
func attempt_movement(to: Vector2) -> bool:
	var successful := false

	# Clamps the target's x position to the maximum distance
	var to_x_local := to_local(to).x
	to_x_local = clampf(to_x_local
		# Minimum distance
		, -target_forward_max_reach if direction.x == sign(to_x_local) else -target_backwards_max_reach
		# Maximum distance
		, target_forward_max_reach if direction.x == sign(to_x_local) else target_backwards_max_reach)
	
	# Checks if the target position is a valid position
	ground_raycast.target_position.y = leg_maximum_height
	ground_raycast.position.x = to_x_local
	ground_raycast.force_raycast_update()

	if ground_raycast.is_colliding():
		# Sets the target as top level, so it doesn't move along with the leg
		target.top_level = true

		var desired_position = ground_raycast.get_collision_point()

		# Rotate magnets to match the ground's normal
		rotate_magnets_with_normal(ground_raycast.get_collision_normal())
		reposition_target(desired_position)
		successful = true
	
	# Resets the raycast's position
	#ground_raycast.position.x = 0

	return successful

## Rotates the magnets to match the given normal.
func rotate_magnets_with_normal(normal: Vector2) -> void:
	var normal_angle := Vector2.UP.angle_to(normal)

	rotate_magnets(normal_angle)

## Rotates the magnets by the given angle in radians.
func rotate_magnets(angle_in_radians: float) -> void:
	for index in limbs.size():
		FABRIK_modification.set_fabrik_joint_magnet_position(index,
			limbs[index].get_meta("default_magnet").rotated(angle_in_radians))


## Plays a tween to reposition the leg's target to the given position.
func reposition_target(new_position: Vector2, speed_multiplier := 1.0) -> void:
	var tween := create_tween()
	tween.tween_property(target, "global_position:y", global_position.y + Vector2.UP.rotated(global_rotation).y, limb_reposition_speed * 0.2 * speed_multiplier)
	tween.tween_property(target, "global_position:x", new_position.x, limb_reposition_speed * speed_multiplier)
	tween.chain().tween_property(target, "global_position:y", new_position.y, limb_reposition_speed * 0.2 * speed_multiplier)

## Switches behavior to background mode.
func set_background_mode(active: bool) -> void:
	for limb in limbs:
		limb.get_meta("sprite").modulate = (Color.BLACK if active else Color.WHITE)
	
	show_behind_parent = active

## Returns the marker to a default position and disables the target's node's top level property.
func rest() -> void:
	reposition_target(global_position + default_target_position.rotated(global_rotation), limb_reposition_speed * 10)
	target.top_level = false
