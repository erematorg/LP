extends Node
class_name InteractionManager

signal entity_interaction(entity1, entity2)

@export var interaction_distance: float = 50.0  # Exposed to editor

var interaction_cooldown: float = 1.0
var time_since_last_interaction: float = 0.0

func _ready():
	set_process(true)

func _process(delta):
	time_since_last_interaction += delta
	if time_since_last_interaction >= interaction_cooldown:
		check_interactions()
		time_since_last_interaction = 0.0

# Check for interactions between entities
func check_interactions():
	var entities = get_tree().get_nodes_in_group("entities")
	for i in range(entities.size()):
		var entity1 = entities[i]
		for j in range(i + 1, entities.size()):
			var entity2 = entities[j]
			if entity1.is_inside_tree() and entity2.is_inside_tree():
				var distance = entity1.global_position.distance_to(entity2.global_position)
				if distance < interaction_distance:
					interact(entity1, entity2)

# Emit interaction signal between two entities
func interact(entity1: Node2D, entity2: Node2D):
	emit_signal("entity_interaction", entity1, entity2)
	print("Interaction occurred: ", entity1.name, " with ", entity2.name)
