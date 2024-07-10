extends Node2D
class_name Plant

@export var attributes: PlantaeAttributes

var growth_component: GrowthComponent

func _ready():
	growth_component = GrowthComponent.new()
	add_child(growth_component)
	
	if attributes:
		apply_entity_data(attributes)
	add_to_group("entities")

	var interaction_manager = get_node("/root/SIFDemo/InteractionManager")
	if interaction_manager:
		interaction_manager.connect("entity_interaction", Callable(self, "_on_entity_interaction"))

func apply_entity_data(data: PlantaeAttributes):
	growth_component.initialize(data)
	introduce_variation()

func introduce_variation():
	for key in attributes.keys():
		if typeof(attributes[key]) == TYPE_FLOAT:
			attributes[key] *= randf_range(0.95, 1.05)
	name = "Plant_" + str(randi())

func _on_entity_interaction(entity1, entity2):
	if self == entity1:
		print("Interacting with", entity2.name)
	elif self == entity2:
		print("Interacting with", entity1.name)
