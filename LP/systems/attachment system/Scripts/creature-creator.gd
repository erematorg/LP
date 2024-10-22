@tool
extends Node
class_name CreatureCreator

var entities : Array[EntityPart]


#We have to connect to the signal from this end
#So we inject the gui, and connect to its spawn signal
func inject_attachment_gui(gui : attachmentgui):
	if gui.spawn_entity.is_connected(new_entity_in_scene):
		print("Signal already properly connected")
		return
	gui.spawn_entity.connect(new_entity_in_scene)
	print("gui injected into creator")


func new_entity_in_scene(entity : EntityPart):
	print("new entity! - creator")
	entities.push_back(entity)


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	entities = []


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
