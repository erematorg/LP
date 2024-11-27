extends Node
class_name MovementComponent

@export var movement_type: String = "walk"
@export var speed: float = 10.0
@export var acceleration: float = 500.0 ### Added by Lim
@export var stamina: float = 100.0

### Added by Lim
# determine the creature's maximum speed based on its number of limbs
@onready var maximum_speed: float = speed * get_parent().get_parent().get_bone_count()

func initialize(data):
	speed = data.speed
	stamina = data.stamina
	movement_type = data.movement_type

### Added by Lim
func _process(delta):
	# dynamic recalculation of speed based on number of limbs
	maximum_speed = speed * get_parent().get_parent().get_bone_count()

### Added By Lim
func _physics_process(delta):
	var x_direction = Input.get_axis("move_left", "move_right")
	var y_direction = Input.get_axis("move_up", "move_down")
	var body: RigidBody2D = get_parent().get_parent().get_parent()
	# if the creature hasn't reached it's maximum speed, apply force to the creature based on the direction of its movement
	if abs(body.linear_velocity.x) < maximum_speed:
		body.apply_central_force(Vector2(x_direction, 0) * acceleration)
	
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
