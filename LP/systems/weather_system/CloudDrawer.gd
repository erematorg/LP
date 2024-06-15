extends Node2D
class_name CloudDrawer
## Clouds are merely a visual representation of
## Humidity.saturated_water_per_area in the current area.

@export var circle_radius:float
@export var circle_sides:int


## returns a list of circles in a different position
## Which can later be fused to draw clouds
func get_random_circles():
	pass

func get_circle():
	var angle=0
