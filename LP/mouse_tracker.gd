@tool
extends Control

var dragging : bool = false

# Make sure to enable this script to run in the editor
func _ready():
	set_process(true)
	set_process_input(true)
	set_process_unhandled_input(true)

# Capture input events
func _input(event: InputEvent) -> void:
	print("lmao")
	if Engine.is_editor_hint():
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
			var mouse_pos = get_viewport().get_mouse_position()
			print(mouse_pos)
	if event is InputEventMouseButton:
		var mouse_event = event as InputEventMouseButton
		if mouse_event.button_index == MOUSE_BUTTON_LEFT:
			if mouse_event.pressed:
				print("Left mouse button pressed in editor")
			else:
				print("Left mouse button released in editor")
	elif event is InputEventMouseMotion:
		print("Mouse motion detected in editor at: ", event.position)

func _process(delta):
	if Engine.is_editor_hint():
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
			dragging = true
		else:
			if dragging:
				#This happens right as we release the left mouse button
				print("release")
			dragging = false
