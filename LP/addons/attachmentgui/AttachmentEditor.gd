@tool
extends EditorPlugin
class_name AttachmentEditor

var dock
var editor : EditorInterface
var entities_folder : String = "res://entities/"
var parts_folder : String = "res://entities/parts/"
var components_folder : String = "res://entities/components/"
var cosmetics_folder : String = "res://entities/cosmetics/"


func _enter_tree() -> void:
	editor = get_editor_interface()
	dock = preload("res://addons/attachmentgui/Scenes/AttachmentGUIDock.tscn").instantiate()
	add_control_to_dock(DOCK_SLOT_LEFT_UL, dock)
	dock.attachment_editor = self
	if not dock:
		push_error("Failed to load AttachmentGUI dock!")


func _exit_tree() -> void:
	remove_control_from_docks(dock)
	dock.free()


## Editor Interactions
func edit_scene(object : PackedScene, path : String):
	if path == "" or not object:
		push_error("Invalid scene path or object!")
		return
	editor.open_scene_from_path(path)
	editor.get_editor_viewport_2d()


func get_open_scene() -> Node:
	return editor.get_edited_scene_root()


func load_resources_from_folder(receiver: AttachmentGui, target: String, folder_path: String):
	var dir = DirAccess.open(folder_path)
	var dock_gui = receiver

	# Early exit if any critical argument is null or directory is invalid
	if receiver == null or dock_gui == null or not dir:
		push_error("Dock/Receiver/FolderPath is null or invalid - AttachmentEditor")
		return
	
	dir.list_dir_begin()
	var file_name = dir.get_next()
	var files_found = 0

	# Loop through the directory entries
	while file_name != "":
		# Avoid processing special entries
		if file_name == "." or file_name == "..":
			file_name = dir.get_next()
			continue

		var file_path = folder_path + "/" + file_name

		if dir.current_is_dir():
			# Recursive call for directories
			var secondary_dir = DirAccess.open(file_path)
			if secondary_dir and secondary_dir.get_files().size() > 0:
				receiver.add_grid_label(file_name + ":")  # Add folder name as a label
			load_resources_from_folder(dock_gui, target, file_path)
		else:
			# Process only files that match the target resource type
			if file_name.ends_with(target):
				receiver.add_resource_item(file_path, file_name)
				files_found += 1

		# Move to the next file
		file_name = dir.get_next()
	dir.list_dir_end()

	# Log the number of found files
	print("Found " + str(files_found) + " file(s) in " + folder_path)
	
	# If no files were found in a subfolder, add an "-Empty-" label
	if files_found < 1 and folder_path != folder_path:
		receiver.add_grid_label("-Empty-", false)
