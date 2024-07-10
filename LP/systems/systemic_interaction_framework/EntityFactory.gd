extends Node
class_name EntityFactory

static func create_entity(type: String, category: String, data: Dictionary) -> Node2D:
	var entity
	match category:
		"animals":
			entity = preload("res://scenes/base_scenes/Animal.tscn").instantiate()
		"plants":
			entity = preload("res://scenes/base_scenes/Plant.tscn").instantiate()
		# Add more categories if needed

	if entity:
		entity.set("attributes", data)
		entity.name = type + "_" + str(randi())
		return entity
	return null
