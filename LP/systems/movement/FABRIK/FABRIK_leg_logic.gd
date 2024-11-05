## Used to control the movement of a leg.
class_name FABRIKLegLogic
extends Resource

## Contains the template to generate the legs. It will be used if the leg has no default information defined.
@export var leg_template: Array[FABRIKLegInfo]

## The maximum distance the leg can reach when moving along the defined direction.
@export var target_forward_max_reach := 20.0

## The maximum distance the leg can reach when moving opposite to the defined direction.
@export var target_backwards_max_reach := 10.0

## The speed at which the leg repositions itself to the target. Useful for representing emotions (ex: fear, anger, calm, etc.)
@export var limb_reposition_speed := 0.1

## The maximum height the leg can reach.
@export var leg_maximum_height := 0.0

## A target position that an entity should move towards.
var target_location: Vector2

## The direction that the creature is facing.
var creature_direction: Vector2

## A list of all the legs that are attached to this logic component.
var leg_references: Array[FABRIKLeg]

## Attaches the logic component to the leg.
func attach_to_leg(leg: FABRIKLeg) -> void:
	leg.leg_logic = self

	leg.touched_ground.connect(on_floor_collision)
	leg.left_ground.connect(on_floor_left)

	# If the leg has no information defined, use the template.
	if leg.FABRIK_limbs_info.is_empty():
		leg.FABRIK_limbs_info = leg_template
	
	# If the leg allows the logic component to set the reach values, set them.
	if leg.use_logic_component_stat_values:
		leg.target_forward_max_reach = target_forward_max_reach
		leg.target_backwards_max_reach = target_backwards_max_reach
		leg.limb_reposition_speed = limb_reposition_speed
		leg.leg_maximum_height = leg_maximum_height

	leg_references.append(leg)
	

## Called by the leg. Should be overriden.
func leg_callback(leg: FABRIKLeg) -> void:
	leg.direction = creature_direction

	if leg.is_on_floor and leg.is_target_out_of_range():
		var random_offset := Vector2(randf_range(-1, 1), randf_range(-1, 1) * 20)
		leg.attempt_movement(target_location + random_offset)

## Called when leg lands on the floor. Should be overriden.
func on_floor_collision(leg: FABRIKLeg) -> void:
	pass

## Called when leg leaves the floor. Should be overriden.
func on_floor_left(leg: FABRIKLeg) -> void:
	pass
