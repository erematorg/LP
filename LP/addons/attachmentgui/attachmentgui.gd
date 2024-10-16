@tool
extends EditorPlugin
class_name attachmentgui

var dock

func _enter_tree() -> void:
	dock = preload("res://addons/attachmentgui/AttachmentGUIDock.tscn").instantiate()
	add_control_to_dock(DOCK_SLOT_LEFT_UL, dock)
	dock.attachment_gui = self

func _init() -> void:
	self.name = "Attachment GUI"

func _exit_tree() -> void:
	remove_control_from_docks(dock)
	dock.free()

## Editor Interactions
func edit_scene(object : PackedScene, path : String):
	var editor = get_editor_interface()
	var err = ResourceSaver.save(object, path)
	if err == OK:
		editor.open_scene_from_path(path)
	else:
		("Error saving creature!")
