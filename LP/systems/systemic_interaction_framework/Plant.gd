extends Node2D
class_name Plant

@export var attributes: PlantaeAttributes

var growth_component: GrowthComponent

func _ready():
	# Initialize growth component only if needed
	if not growth_component:
		growth_component = GrowthComponent.new()
		add_child(growth_component)
	
	# Apply attributes if they are provided
	if attributes:
		apply_entity_data(attributes)
	
	# Add this entity to the "entities" group for interactions
	add_to_group("entities")

	# Connect to the interaction signal in the InteractionManager
	var interaction_manager = get_node("/root/SIFDemo/InteractionManager")
	if interaction_manager:
		interaction_manager.connect("entity_interaction", Callable(self, "_on_entity_interaction"))

# Apply entity data to the plant, including its growth attributes
func apply_entity_data(data: PlantaeAttributes):
	growth_component.initialize(data)
	introduce_variation()

# Introduce some variation to the plant's attributes
func introduce_variation():
	for key in attributes.keys():
		if typeof(attributes[key]) == TYPE_FLOAT:
			attributes[key] *= randf_range(0.95, 1.05)
	name = "Plant_" + str(randi())

# Handle interactions with other entities
func _on_entity_interaction(entity1, entity2):
	if self == entity1:
		print("Plant interacting with", entity2.name)
	elif self == entity2:
		print("Plant interacting with", entity1.name)
