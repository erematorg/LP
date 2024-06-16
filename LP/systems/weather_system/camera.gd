extends Camera2D

@export var movement_speed:float

func _process(delta):
	var direction=Input.get_vector("move_left","move_right","move_up","move_down")
	position+=direction*movement_speed*delta/zoom
