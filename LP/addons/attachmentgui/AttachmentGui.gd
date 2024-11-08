@tool
extends Control
class_name AttachmentGui

signal spawn_entity(entity)
signal spawn_socket(socket)
signal spawn_component(component)
signal spawn_cosmetic(cosmetic)

var attachment_editor : AttachmentEditor
var current_creature_scene : PackedScene
@export var TEMPLATE_SCENE : PackedScene
@export var new_button : Button
@export var edit_button : Button
@export var socket_button : Button
@export var file_dialog : FileDialog
@export var select_dialog : FileDialog
@export var path_label : Label
@export var current_scene_label : Label

#Body parts / Entitites / Cosmetics
var active_container
@onready var parts_panel: Panel = $PartsPanel
@export var parts_container : BoxContainer
@onready var components_panel: Panel = $ComponentsPanel
@export var components_container : BoxContainer
@onready var cosmetics_panel: Panel = $CosmeticsPanel
@export var cosmetics_container : BoxContainer

const LIMB_SOCKET = preload("res://systems/attachment system/limb_socket.tscn")
const ATTACHMENT_GUI_MAINLABEL = preload("res://addons/attachmentgui/attachment_gui_mainlabel.tres")
const ATTACHMENT_GUI_SMALLLABEL = preload("res://addons/attachmentgui/attachment_gui_smalllabel.tres")
const BLACKOUTLINEARM = preload("res://addons/attachmentgui/Sprites/blackoutlinearm.png")
const DNA = preload("res://addons/attachmentgui/Sprites/dna.png")


func _ready() -> void:
	path_label.text = "PATH:"
	current_scene_label.text = "SCENE:"
	edit_button.disabled = true
	socket_button.disabled = false
	parts_panel.visible = false
	components_panel.visible = false
	cosmetics_panel.visible = false


func ensure_components():
	if attachment_editor == null:
		push_error("Warning: No reference to attachmentGUI")
	if TEMPLATE_SCENE == null:
		push_error("Warning: No reference to TEMPLATE_SCENE")
	if new_button == null:
		push_error("Warning: No reference to new_button")
	if edit_button == null:
		push_error("Warning: No reference to edit_button")
	if file_dialog == null:
		push_error("Warning: No reference to file_dialog")
	if path_label == null:
		push_error("Warning: No reference to path_label")
	if current_scene_label == null:
		push_error("Warning: No reference to current_scene_label")
	if parts_container == null:
		push_error("Warning: No reference to parts_container")
	if components_container == null:
		push_error("Warning: No reference to components_container")
	if cosmetics_container == null:
		push_error("Warning: No reference to cosmetics_container")


## Open the file dialog to select/create a new creature scene
func _on_new_button_pressed() -> void:
	ensure_components()
	file_dialog.popup_centered()


## Selecting a creature scene!
func _on_select_button_pressed() -> void:
	ensure_components()
	select_dialog.popup_centered()


## When pressing 'Edit' open the new template scene
func _on_edit_button_pressed() -> void:
	ensure_components()
	clear_containers()
	if !path_label.text.ends_with(".tscn"):
		path_label.text = path_label.text+".tscn"
	if not current_creature_scene:
		push_error("current_creature_scene is null!")
		return
	attachment_editor.edit_scene(current_creature_scene, path_label.text)
	var rootNode = attachment_editor.get_open_scene()
	if rootNode == null or rootNode is not CreatureCreator:
		print("Root is either null or not creaturecreator - attachmentgui")
		current_scene_label.text = "NO SCENE FOUND"
		return
	current_scene_label.text = str(rootNode.name)
	## Load all resources from folder one after another into the "active" container
	active_container = parts_container
	attachment_editor.load_resources_from_folder(self, ".tscn", attachment_editor.parts_folder) #load parts
	active_container = components_container
	attachment_editor.load_resources_from_folder(self, ".gd", attachment_editor.components_folder) #load components
	active_container = cosmetics_container
	attachment_editor.load_resources_from_folder(self, ".tscn", attachment_editor.cosmetics_folder) #load cosmetics
	rootNode.inject_attachment_gui(self)
	components_panel.visible = true
	socket_button.disabled = false


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


func select_creature(path : String):
	path_label.text = path
	var packedscene = load(path)
	var scene = packedscene.instantiate()
	print(path) # string path
	print(packedscene) #packed scene
	print(scene) #root of scene
	if scene:
		current_creature_scene = packedscene
		edit_button.disabled = false
		print("New creature scene selected!")
	else:
		print("Error with selecting creature")


# Function to add a resource item to the container, called from attachmenteditor
func add_resource_item(file_path: String, file_name : String):
	#Create our button
	if file_path == "":
		push_error("Path is null")
		return
	var button = Button.new()
	button.vertical_icon_alignment = VERTICAL_ALIGNMENT_CENTER
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.size_flags_vertical = Control.SIZE_EXPAND_FILL
	button.tooltip_text = file_path
	add_to_container(file_path, button, file_name)
	change_button_icon_and_text(button, file_path, file_name)


func add_to_container(path, button : Button, name):
	# Connect the button to a function that will handle instantiating the resource
	if active_container == parts_container:
		button.pressed.connect(self.resource_button_pressed.bind(path))
	elif active_container == cosmetics_container:
		button.pressed.connect(self.add_cosmetic_instance.bind(path))
	else:
		button.pressed.connect(self.add_component_instance.bind(path))
	active_container.add_child(button)


func change_button_icon_and_text(button : Button, path : String, name : String):
	var file_scene = load(path)
	button.text = name
	var part_preview
	var instance
	if path.ends_with(".tscn"):
		instance = file_scene.instantiate()
	#Determine what we are adding depending on the active container
	match active_container:
		parts_container:
			if instance is EntityPart:
				part_preview = instance.thumbnail
				var key = EntityPart.type.keys()[EntityPart.type.values().find(instance.entity_type)]
				button.text = instance.preview_name + " : \n" + key
				button.icon = part_preview if part_preview else BLACKOUTLINEARM

		components_container:
			button.icon = DNA

		cosmetics_container:
			if instance is Sprite2D:
				button.icon = instance.texture


#Adding a new label means we are entering a new subfolder or showing "Empty"
func add_grid_label(label, header : bool = true):
	var label_to_add = Label.new()
	active_container.add_child(label_to_add)
	if header:
		label_to_add.label_settings = ATTACHMENT_GUI_MAINLABEL
	else:
		label_to_add.label_settings = ATTACHMENT_GUI_SMALLLABEL
	label_to_add.text = str(label)


# When a part button is clicked
func resource_button_pressed(resource: String):
	if not get_tree().edited_scene_root.name == current_scene_label.text:
		push_warning("No longer in the correct scene! Switch back to creature create scene or restart creature create process!")
		return
	var instance = load(resource)
	var new_instance_scene : Node2D = instance.instantiate()
	if not new_instance_scene:
		push_error("instance scene is null!")
		return
	spawn_entity.emit(new_instance_scene)


func add_component_instance(resource: String):
	if not get_tree().edited_scene_root.name == current_scene_label.text:
		push_warning("No longer in the correct scene! Switch back to creature create scene or restart creature create process!")
		return
	spawn_component.emit(resource)


func add_cosmetic_instance(resource : String):
	if not get_tree().edited_scene_root.name == current_scene_label.text:
		push_warning("No longer in the correct scene! Switch back to creature create scene or restart creature create process!")
		return
	var instance = load(resource)
	var sprite : Node2D = instance.instantiate()
	if not sprite:
		push_error("instance scene is null!")
		return
	spawn_cosmetic.emit(sprite)


func clear_containers():
	var children = parts_container.get_children() + components_container.get_children()
	for child in children:
		child.free()


func _on_socket_button_pressed() -> void:
	var new_socket = LIMB_SOCKET.instantiate()
	if not new_socket:
		push_error("Socket instantiation failed!")
		return
	spawn_socket.emit(new_socket)


##Selecting body parts or components to show!
func _on_selection(index: int) -> void:
	match index:
		0:
			components_panel.visible = true
			parts_panel.visible = false
			cosmetics_panel.visible = false
		1: 
			parts_panel.visible = true
			components_panel.visible = false
			cosmetics_panel.visible = false
		2:
			parts_panel.visible = false
			components_panel.visible = false
			cosmetics_panel.visible = true
