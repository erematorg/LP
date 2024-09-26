extends Node2D
class_name Animal

@export var attributes: AnimaliaAttributes

var movement_component: MovementComponent
var health_component: HealthComponent
var dna_component: DNAComponent

func _ready():
	# Initialize components only if needed
	initialize_components()
	
	# Apply attributes if they are provided
	if attributes:
		apply_entity_data(attributes)

	# Add this entity to the "entities" group for interactions
	add_to_group("entities")

	# Connect to the interaction signal in the InteractionManager
	var interaction_manager = get_node("/root/SIFDemo/InteractionManager")
	if interaction_manager:
		interaction_manager.connect("entity_interaction", Callable(self, "_on_entity_interaction"))

# Initialize all required components if they are not already initialized
func initialize_components():
	if not movement_component:
		movement_component = MovementComponent.new()
		add_child(movement_component)
	
	if not health_component:
		health_component = HealthComponent.new()
		add_child(health_component)
	
	if not dna_component:
		dna_component = DNAComponent.new()
		add_child(dna_component)

# Apply entity data to the animal, including its movement, health, and DNA attributes
func apply_entity_data(data: AnimaliaAttributes):
	movement_component.initialize(data)
	health_component.initialize(data)
	dna_component.initialize(data.parent1_attributes, data.parent2_attributes)
	introduce_variation()

# Introduce some variation to the animal's attributes
func introduce_variation():
	for key in attributes.keys():
		if typeof(attributes[key]) == TYPE_FLOAT:
			attributes[key] *= randf_range(0.95, 1.05)
	name = "Animal_" + str(randi())

# Handle interactions with other entities
func _on_entity_interaction(entity1, entity2):
	if self == entity1:
		print("Animal interacting with", entity2.name)
	elif self == entity2:
		print("Animal interacting with", entity1.name)
