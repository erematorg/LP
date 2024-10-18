extends Node 

#Misc script to move a marker/sprite along with the mouse
#This has been used to test the IK of entitites while playing a scene
#It can be used for anything else

@export var target_marker : Marker2D

func _input(event):
	if event is InputEventMouseMotion or event is InputEventMouseButton:
		target_marker.position = target_marker.get_parent().get_local_mouse_position()# mouse_pos
