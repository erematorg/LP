@tool
extends EditorPlugin
class_name attachmentgui

var dock
var editor : EditorInterface
var entities_folder : String = "res://Entities/"

func _enter_tree() -> void:
	editor = get_editor_interface()
	dock = preload("res://addons/attachmentgui/Scenes/AttachmentGUIDock.tscn").instantiate()
	add_control_to_dock(DOCK_SLOT_LEFT_UL, dock)
	dock.attachment_gui = self

func _init() -> void:
	self.name = "Attachment GUI"

func _exit_tree() -> void:
	remove_control_from_docks(dock)
	dock.free()

## Editor Interactions
func edit_scene(object : PackedScene, path : String):
	print("Path to new creature: " + path)
	print("New creature : " + str(object.resource_name))
	editor.open_scene_from_path(path)
	editor.get_editor_viewport_2d()
		
func get_open_scene() -> Node:
	return editor.get_edited_scene_root()

func load_resources_from_folder(attachment_gui):
	var dir = DirAccess.open(entities_folder)
	if not dir:
		print("Something went wrong opening entities folder")
		return
		
	dir.list_dir_begin()
	var file_name = dir.get_next()
	
	# Loop through the directory entries
	while file_name != "":
		if !dir.current_is_dir():
			var file_path = entities_folder + file_name
			# Check if the file is a resource we care about (like scenes, textures, etc.)
			if file_name.ends_with(".tscn"):
				attachment_gui.add_resource_item(file_path, file_name)
				print("Found scene: " + file_path)
		file_name = dir.get_next()
	dir.list_dir_end()
