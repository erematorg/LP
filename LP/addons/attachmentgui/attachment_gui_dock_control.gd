@tool
extends Control
class_name attachmentgui

signal spawn_entity(entity)
signal spawn_socket(socket)

var attachment_editor : attachmenteditor
var current_creature_scene : PackedScene
@export var TEMPLATE_SCENE : PackedScene
@export var new_button : Button
@export var edit_button : Button
@export var socket_button : Button
@export var file_dialog : FileDialog
@export var path_label : Label
@export var current_scene_label : Label
@export var item_container : BoxContainer
@onready var parts_panel: Panel = $PartsPanel
const LIMB_SOCKET = preload("res://systems/attachment system/limb_socket.tscn")
const ATTACHMENT_GUI_MAINLABEL = preload("res://addons/attachmentgui/attachment_gui_mainlabel.tres")
const ATTACHMENT_GUI_SMALLLABEL = preload("res://addons/attachmentgui/attachment_gui_smalllabel.tres")

func _ready() -> void:
	path_label.text = "PATH:"
	current_scene_label.text = "SCENE:"
	edit_button.disabled = true
	socket_button.disabled = false
	parts_panel.visible = false

func ensure_components():
	if attachment_editor == null:
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
	if !path_label.text.ends_with(".tscn"):
		path_label.text = path_label.text+".tscn"
	attachment_editor.edit_scene(current_creature_scene, path_label.text)
	var rootNode = attachment_editor.get_open_scene()
	if rootNode == null:
		print("No root node in scene")
		current_scene_label.text = "NO SCENE FOUND"
		return
	else:
		current_scene_label.text = str(rootNode.name)
	attachment_editor.load_resources_from_folder(self)
	enable_parts_panel()
	if rootNode is CreatureCreator:
		rootNode.inject_attachment_gui(self)


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
		save_new_creature()
	else:
		print("Error with creating creature" + result)
		return


func enable_parts_panel():
	parts_panel.visible = true
	socket_button.disabled = false
	

func save_new_creature():
	var savelabel
	if path_label.text.ends_with(".tscn"):
		savelabel = ""
	else:
		savelabel = ".tscn"
	var error = ResourceSaver.save(current_creature_scene, path_label.text+savelabel)
	if error == OK:
		print("Scene saved successfully at: ", path_label.text)
	else:
		print("Failed to save scene. Error code: ", error)


# Function to add a resource item to the container
func add_resource_item(file_path: String, file_name : String):
	#Create our button
	var button = Button.new()
	button.vertical_icon_alignment = VERTICAL_ALIGNMENT_CENTER
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.size_flags_vertical = Control.SIZE_EXPAND_FILL
	button.tooltip_text = file_path
	# Connect the button to a function that will handle instantiating the resource
	button.pressed.connect(self.resource_button_pressed.bind(file_path))
	item_container.add_child(button) #Add button to the grid container
	change_button_icon_and_text(button, file_path, file_name)


func change_button_icon_and_text(button : Button, path : String, name : String):
	var file_scene = load(path)
	var part_preview
	var instance = file_scene.instantiate()
	#If scene is an EntityPart, try to retrieve its thumbnail
	if instance is EntityPart:
		part_preview = instance.thumbnail
		button.text = instance.preview_name
	else:
		button.text = name
	#If thumbnail is valid, use that or use file name
	if part_preview:
		button.icon = part_preview



#Adding a new label means we are entering a new subfolder or showing "Empty"
func add_grid_label(label, header : bool = true):
	var label_to_add = Label.new()
	item_container.add_child(label_to_add)
	if header:
		label_to_add.label_settings = ATTACHMENT_GUI_MAINLABEL
	else:
		label_to_add.label_settings = ATTACHMENT_GUI_SMALLLABEL
	label_to_add.text = str(label)


# When a resource button is clicked, this will instantiate the resource in the scene
func resource_button_pressed(resource: String):
	if not get_tree().edited_scene_root.name == current_scene_label.text:
		print("No longer in the correct scene! Switch back to creature create scene or restart creature create process!")
		return
	var instance = load(resource)
	var new_instance_scene : Node = instance.instantiate()
	print(get_tree().edited_scene_root)
	var cc : CreatureCreator = get_tree().edited_scene_root
	cc.creature_root.add_child(new_instance_scene)
	#get_tree().edited_scene_root.add_child(new_instance_scene)
	new_instance_scene.owner = get_tree().edited_scene_root
	print("Instantiated resource: ", resource)
	spawn_entity.emit(new_instance_scene)
	
	
	
func clear_container():
	var children = item_container.get_children()
	for child in children:
		child.free()


func _on_socket_button_pressed() -> void:
	var new_socket = LIMB_SOCKET.instantiate()
	get_tree().edited_scene_root.add_child(new_socket)
	new_socket.owner = get_tree().edited_scene_root
	print("Instantiated new socket: ", new_socket.name)
	spawn_socket.emit(new_socket)
