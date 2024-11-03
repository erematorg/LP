@tool
extends Node
class_name MouseTracker

signal stopped_dragging
var dragging : bool = false
var drag_time
const max_drag_time = 0.2

## If we hold the left mouse button down for more than max_drag_time
## Then that means we are dragging the mouse and probably something with it in the scene
## This script is responsible for just tracking that action, letting us drag and drop items in the scene

func _process(delta):
	if not Engine.is_editor_hint():
		return
		
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		drag_time+=delta
		if drag_time >= max_drag_time and not dragging:
			print("We are dragging the mouse")
			dragging = true
	else:
		if dragging:
			#This happens right as we release the left mouse button
			stopped_dragging.emit()
		dragging = false
		drag_time = 0
