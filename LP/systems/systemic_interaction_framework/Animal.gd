extends Node2D
class_name Animal

@export var attributes: AnimaliaAttributes

var movement_component: MovementComponent
var health_component: HealthComponent
var dna_component: DNAComponent

func _ready():
	initialize_components()
	if attributes:
		apply_entity_data(attributes)
	add_to_group("entities")

	var interaction_manager = get_node("/root/SIFDemo/InteractionManager")
	if interaction_manager:
		interaction_manager.connect("entity_interaction", Callable(self, "_on_entity_interaction"))

func initialize_components():
	movement_component = MovementComponent.new()
	add_child(movement_component)
	health_component = HealthComponent.new()
	add_child(health_component)
	dna_component = DNAComponent.new()
	add_child(dna_component)

func apply_entity_data(data: AnimaliaAttributes):
	movement_component.initialize(data)
	health_component.initialize(data)
	dna_component.initialize(data.parent1_attributes, data.parent2_attributes)
	introduce_variation()

func introduce_variation():
	for key in attributes.keys():
		if typeof(attributes[key]) == TYPE_FLOAT:
			attributes[key] *= randf_range(0.95, 1.05)
	name = "Animal_" + str(randi())

func _on_entity_interaction(entity1, entity2):
	if self == entity1:
		print("Interacting with", entity2.name)
	elif self == entity2:
		print("Interacting with", entity1.name)
