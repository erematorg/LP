@tool
extends Control

signal create(name: String, script_content: String)

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	%CreateButton.pressed.connect(_on_create_button_pressed)

func _on_create_button_pressed():
	var name: String = %TextEdit.text
	create.emit(name, %ScriptContentLb.text % name)
