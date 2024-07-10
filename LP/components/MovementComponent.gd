extends Node
class_name MovementComponent

@export var movement_type: String = "walk"
@export var speed: float = 1.0
@export var stamina: float = 100.0

func initialize(data):
	speed = data.speed
	stamina = data.stamina
	movement_type = data.movement_type

func move(direction: Vector2):
	if stamina <= 0:
		print("Entity is too tired to move")
		return

	var parent_node = get_parent()  # Get the parent node (e.g., Animal)
	if parent_node:
		match movement_type:
			"walk":
				parent_node.global_position += direction * speed
			"fly":
				parent_node.global_position += direction * speed * 1.5
			"swim":
				parent_node.global_position += direction * speed * 0.5

	stamina -= 1.0

func change_movement_type(new_type: String):
	movement_type = new_type if new_type in ["walk", "fly", "swim"] else movement_type
