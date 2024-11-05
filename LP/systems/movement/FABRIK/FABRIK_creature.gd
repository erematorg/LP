class_name FABRIKCreature
extends Node2D

## The leg logic component that will be attached to all legs.
@export var leg_logic: FABRIKLegLogic

## The direction the creature is facing.
var direction: Vector2 = Vector2.RIGHT


## Initializes the creature and the legs attached to it.
func _ready():
	# Makes the leg logic component unique to this creature.
	leg_logic = leg_logic.duplicate()

	var leg_array := fetch_legs_in(self)

	apply_leg_logic(leg_array)
	activate_legs(leg_array)


func _process(_delta):
	if is_instance_valid(leg_logic):
		leg_logic.target_location = global_position + (Vector2.RIGHT * 5).rotated(global_rotation)
		leg_logic.creature_direction = direction


# ------- LEG GENERATION -------

## Looks for leg nodes and returns them in an array.
func fetch_legs_in(node: Node) -> Array:
	var leg_array: Array = []

	for child in node.get_children():

		# Adds the child to the array if it is a leg.
		if child is FABRIKLeg: leg_array.append(child)

		# Recursively searches for legs in the child.
		if child.get_child_count() > 0:
			leg_array += fetch_legs_in(child)
	
	return leg_array

## Attaches the leg logic component to all legs.
func apply_leg_logic(leg_array) -> void:
	for leg in leg_array:
		leg_logic.attach_to_leg(leg)

## Activates all legs.
func activate_legs(leg_array: Array) -> void:
	for leg in leg_array:
		leg.generate(leg.FABRIK_limbs_info)
