@tool
extends Control

var attachment_gui : attachmentgui
var current_creature_scene : PackedScene
@export var TEMPLATE_SCENE : PackedScene
@export var new_button : Button
@export var edit_button : Button
@export var file_dialog : FileDialog
@export var path_label : Label
@export var current_scene_label : Label
@export var item_container : GridContainer

func _ready() -> void:
	path_label.text = "PATH:"
	current_scene_label.text = "SCENE:"
	edit_button.disabled = true

func ensure_components():
	if attachment_gui == null:
		print("Warning: No reference to attachmentGUI")
	if TEMPLATE_SCENE == null:
		print("Warning: No reference to TEMPLATE_SCENE")
	if new_button == null:
		print("Warning: No reference to new_button")
	if edit_button == null:
		print("Warning: No reference to edit_button")
	if file_dialog == null:
		print("Warning: No reference to file_dialog")
	if path_label == null:
		print("Warning: No reference to path_label")
	if current_scene_label == null:
		print("Warning: No reference to current_scene_label")
	if item_container == null:
		print("Warning: No reference to item_container")


#Open the file dialog to select/create a new creature scene
func _on_new_button_pressed() -> void:
	file_dialog.popup_centered()
	ensure_components()


#When pressing 'Edit' open the new template scene
func _on_edit_button_pressed() -> void:
	ensure_components()
	clear_container()
	attachment_gui.edit_scene(current_creature_scene, path_label.text)
	var rootNode = attachment_gui.get_open_scene()
	if rootNode == null:
		print("No root node in scene")
		current_scene_label.text = "NO SCENE FOUND"
		return
	else:
		current_scene_label.text = str(rootNode.name)
	attachment_gui.load_resources_from_folder(self)
	

#When selecting a path for the new creature, save it
#and allow us to edit that scene
func _on_file_dialog_file_selected(path: String) -> void:
	path_label.text = path
	#Error handling
	if TEMPLATE_SCENE == null:
		print("TEMPLATE_SCENE is null! Make sure it's correctly assigned or preloaded.")
		return
	var creature_instance = TEMPLATE_SCENE.instantiate()
	# Create a new PackedScene to hold the creature
	var scene = PackedScene.new()
	var result = scene.pack(creature_instance)
	if result == OK:
		current_creature_scene = scene
		edit_button.disabled = false
		print("New creature scene created!")
	else:
		print("Error with creating creature" + result)

# Function to add a resource item (like a button or thumbnail) to the container
func add_resource_item(file_path: String, file_name : String):
	var button = Button.new()
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.size_flags_vertical = Control.SIZE_EXPAND_FILL
	# Connect the button to a function that will handle instantiating the resource
	button.pressed.connect(self._on_resource_button_pressed.bind(file_path))
	item_container.add_child(button)
	button.text = file_name +"\n"+ file_path
	
func add_grid_stop():
	pass

# When a resource button is clicked, this will instantiate the resource in the scene
func _on_resource_button_pressed(resource: String):
	if not get_tree().edited_scene_root.name == current_scene_label.text:
		print("No longer in the correct scene! Switch back to creature create scene or restart creature create process!")
		return
	var instance = load(resource)
	var new_instance_scene : Node = instance.instantiate()
	print(get_tree().edited_scene_root)
	#attachment_gui.get_open_scene().get_tree().edited_scene_root.add_child(new_instance_scene)
	get_tree().edited_scene_root.add_child(new_instance_scene)
	new_instance_scene.owner = get_tree().edited_scene_root
	print("Instantiated resource: ", resource)
	
func clear_container():
	for i in item_container.get_child_count():
		item_container.get_child(i).queue_free()
