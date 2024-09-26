extends Node
class_name EntityFactory

# Creates an entity based on type, category, and provided data
static func create_entity(type: String, category: String, data: Dictionary) -> Node2D:
	var entity
	
	# Match the category to determine which entity to create
	match category:
		"animals":
			entity = preload("res://scenes/base_scenes/Animal.tscn").instantiate()
		"plants":
			entity = preload("res://scenes/base_scenes/Plant.tscn").instantiate()
		_:
			# If the category doesn't match any predefined types, log an error and return null
			print("Entity creation failed: Undefined category '" + category + "'.")
			return null
	
	# If the entity was successfully created
	if entity:
		# Set the entity's attributes from the provided data dictionary
		entity.set("attributes", data)
		# Give the entity a unique name by appending a random integer to its type
		entity.name = type + "_" + str(randi())
		# Log successful creation
		print("Entity created: Type=" + type + ", Category=" + category)
		return entity
	else:
		# Log failure if no entity was created
		print("Entity creation failed for type: '" + type + "' in category: '" + category + "'.")
	
	return null
