extends Node
class_name EntityLoader

var entity_registry = {}
var entities = {}

func _ready():
	# Update entities JSON if needed
	if not update_entities_json():
		load_entity_data()
	load_entities()

# Load entity data from JSON file
func load_entity_data():
	entity_registry = load_from_json("res://data/entities/entities.json")
	if entity_registry.size() > 0:
		print("Entity data loaded successfully: ", entity_registry)
	else:
		print("Error loading JSON")

# Update entities JSON file if there are changes
func update_entities_json() -> bool:
	var entities_data = {
		"animals": [],
		"plants": []
	}
	
	var data_path = "res://data/entities/"
	var dir = DirAccess.open(data_path)
	var updated = false
	
	if dir:
		dir.list_dir_begin()
		var file_name = dir.get_next()
		while file_name != "":
			if file_name.ends_with(".tres"):
				var resource = load(data_path + file_name)
				if resource is AnimaliaAttributes:
					entities_data["animals"].append({
						"type": file_name.get_basename(),
						"attributes": data_path + file_name
					})
					updated = true
				elif resource is PlantaeAttributes:
					entities_data["plants"].append({
						"type": file_name.get_basename(),
						"attributes": data_path + file_name
					})
					updated = true
			file_name = dir.get_next()
		dir.list_dir_end()
	
	var current_entities_data = load_from_json("res://data/entities/entities.json")
	if current_entities_data != entities_data:
		var file = FileAccess.open("res://data/entities/entities.json", FileAccess.WRITE)
		file.store_string(JSON.stringify(entities_data))
		file.close()
		entity_registry = entities_data
		return true
	entity_registry = current_entities_data
	return false

# Load data from a JSON file
func load_from_json(file_path: String) -> Dictionary:
	var file = FileAccess.open(file_path, FileAccess.READ)
	if file:
		var json_instance = JSON.new()
		var parse_result = json_instance.parse(file.get_as_text())
		file.close()
		if parse_result == OK:
			print("JSON parse successful")
			return json_instance.get_data()
		else:
			print("Error parsing JSON: ", json_instance.get_error_message())
	else:
		print("Error opening file: ", file_path)
	return {}

# Get entity data from the registry based on type and category
func get_entity_data(type: String, category: String) -> Dictionary:
	if not entity_registry.has(category):
		print("Category not found: ", category)
		return {}
	
	var category_data = entity_registry[category]
	for entity in category_data:
		if entity["type"] == type:
			return entity
	
	print("Type not found: ", type)
	return {}

# Load all entities from the registry
func load_entities():
	if entity_registry.size() == 0:
		print("Error: No entities to load.")
		return

	for animal in entity_registry.get("animals", []):
		var entity = EntityFactory.create_entity(animal["type"], "animals", animal)
		if entity:
			entities[animal["type"]] = entity
			add_child(entity)
	for plant in entity_registry.get("plants", []):
		var entity = EntityFactory.create_entity(plant["type"], "plants", plant)
		if entity:
			entities[plant["type"]] = entity
			add_child(entity)

# Get a specific entity based on type and category
func get_entity(type: String, category: String) -> Node2D:
	return entities.get(type, null)
