@tool
extends Control

var attachment_gui : attachmentgui
const TEMPLATE_SCENE = "res://addons/attachmentgui/CreatureCreate.tscn"
var current_creature_scene: PackedScene = null
@export var new_button : Button
@export var edit_button : Button
@export var file_dialog : FileDialog
@export var path_label : Label

func _ready() -> void:
	edit_button.disabled = true


func _on_new_button_pressed() -> void:
	file_dialog.popup_centered()


func _on_edit_button_pressed() -> void:
	attachment_gui.edit_scene(current_creature_scene, path_label.text)


func _on_file_dialog_file_selected(path: String) -> void:
	path_label.text = path
	var creature_template = preload(TEMPLATE_SCENE)
	var creature_instance = creature_template.instantiate()
	# Create a new PackedScene to hold the creature
	current_creature_scene = PackedScene.new()
	current_creature_scene.pack(creature_instance)
	#attachment_gui.edit_scene(current_creature_scene, path)
	edit_button.disabled = false
	print("New creature scene created!")
