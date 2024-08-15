@tool
extends EditorPlugin

const NEW_BT_ACTION_SCENE = preload("res://addons/behaviour_tree_action_creator/NewBTActionScene.tscn")
var inst

var script_content = ""

func _enter_tree() -> void:
	inst = NEW_BT_ACTION_SCENE.instantiate()
	add_control_to_dock(EditorPlugin.DOCK_SLOT_LEFT_UL, inst)
	inst.create.connect(_on_create)

func _on_create(name: String, script_content: String):
	if name == "": return
	
	var script: CSharpScript = CSharpScript.new()
	script.resource_name = name
	script.source_code = script_content
	var result = ResourceSaver.save(script, "res://systems/ai/behaviour_tree/nodes/leaf/" + name + ".cs")
	print(result)

func _exit_tree() -> void:
	remove_control_from_docks(inst)
