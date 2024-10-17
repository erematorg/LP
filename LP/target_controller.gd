extends Node 

@export var target_marker : Marker2D

func _input(event):
	if event is InputEventMouseMotion or event is InputEventMouseButton:
		target_marker.position = target_marker.get_parent().get_local_mouse_position()# mouse_pos
