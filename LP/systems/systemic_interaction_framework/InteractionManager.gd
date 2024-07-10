extends Node
class_name InteractionManager

signal entity_interaction(entity1, entity2)

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
		for j in range(i + 1, entities.size()):
			if entities[i].global_position.distance_to(entities[j].global_position) < 50:
				interact(entities[i], entities[j])

# Emit interaction signal between two entities
func interact(entity1: Node2D, entity2: Node2D):
	emit_signal("entity_interaction", entity1, entity2)
	print("Interacting: ", entity1.name, " with ", entity2.name)
