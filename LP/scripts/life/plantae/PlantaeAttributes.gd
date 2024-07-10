extends Attributes
class_name PlantaeAttributes

@export var height: float = 1.0
## Specifies the color of the plant's flowers, adding visual diversity to the ecosystem.
@export var flower_color: Color = Color(1, 1, 1)
## Represents the number of leaves on the plant, influencing its overall structure.
@export var leaf_count: int = 0

func _init():
	pass
