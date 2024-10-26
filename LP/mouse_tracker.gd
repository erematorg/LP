@tool
extends Node
class_name mouse_tracker

signal stopped_dragging
var dragging : bool = false

func _process(delta):
	if Engine.is_editor_hint():
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
			dragging = true
		else:
			if dragging:
				#This happens right as we release the left mouse button
				stopped_dragging.emit()
			dragging = false
