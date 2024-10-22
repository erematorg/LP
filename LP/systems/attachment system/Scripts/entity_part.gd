extends Node2D
class_name EntityPart

@export var thumbnail : Texture2D
@export var preview_name : String

var is_dragging : bool = false


## Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta: float) -> void:
	#if is_dragging:
		#var mousepos : Vector2 = get_viewport().get_mouse_position()
		#position = mousepos
#
#
#func retrieve_preview() -> Texture2D:
	#return thumbnail
#
#
#func drag_entity():
	#is_dragging = !is_dragging
#
#
#func _on_mouse_entered(viewport: Node, event: InputEvent, shape_idx: int) -> void:
	#if event is InputEventMouseButton:
		#if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
			#drag_entity()
