extends Node

signal saving_started
signal saving_finished
signal saving_failed

signal loading_started
signal loading_failed
signal loading_finished

var load_error := ""
var save_error := ""

var save_path := "user://"
var save_name := "game_save"
var save_extension := ".sav"

var autosave_interval := -1.0
var time := 0.0

## Stores the data loaded from the save file.
var loaded_data: Dictionary = {}

## Counter to limit save_game output
var save_game_call_count := 0

## Called when the node enters the scene tree for the first time.
func _ready():
	print("SaveSystem: _ready() called")
	set_process(false)
	setup_autosave(0.1)
	if save_exists(save_name):
		print("Save file exists, verifying and loading")
		if load_game():
			print("Initial load successful")
		else:
			print("Initial load failed")
			print("Load error: ", load_error)
	else:
		print("Save file does not exist, creating initial save")
		if save_game():
			print("Initial save successful")
		else:
			print("Initial save failed")
			print("Save error: ", save_error)

## Called every frame.
func _process(delta):
	time += delta
	if time >= autosave_interval:
		save_game()
		time = 0.0

## Sets up the autosave timer.
func setup_autosave(interval: float):
	if interval <= 0:
		set_process(false)
		return
	set_process(true)
	autosave_interval = interval
	time = 0.0

## Attempts to load the game.
func load_game(file_name := save_name) -> bool:
	print("load_game() called with file_name: ", file_name)
	save_name = file_name
	loading_started.emit()
	if not save_exists(save_name):
		load_error = "No save file exists."
		print("load_game: ", load_error)
		loading_failed.emit()
		return false
	
	var file = FileAccess.open(get_save_name(), FileAccess.READ)
	if file == null:
		load_error = "Failed to open save file."
		print("load_game: ", load_error)
		loading_failed.emit()
		return false

	var content = file.get_as_text()
	file.close()
	
	print("Save file content: ", content)
	
	if content.is_empty():
		load_error = "Empty save file."
		print("load_game: ", load_error)
		loading_failed.emit()
		return false

	var json = JSON.new()
	var parse_error = json.parse(content)
	
	if parse_error != OK:
		load_error = "Failed to parse save file, error code: " + str(parse_error)
		print("load_game: ", load_error)
		loading_failed.emit()
		return false
	
	var data = json.get_data()
	if typeof(data) != TYPE_DICTIONARY:
		load_error = "Parsed JSON successfully, but result is not a dictionary"
		print("load_game: ", load_error)
		loading_failed.emit()
		return false

	print("Data after parsing: ", data)
	
	if not SaveVersioning.is_data_up_to_date(data):
		print("Save data is not up to date. Updating...")
		var data_converter = SaveVersioning.new()
		data = data_converter.update_data(data)
	
	if data.size() == 0:
		load_error = "Parsed data is empty."
		print("load_game: ", load_error)
		loading_failed.emit()
		return false

	for instance in get_tree().get_nodes_in_group("saveable"):
		if data.has(instance.name):
			var instance_data = data[instance.name]
			load_instance(instance, instance_data)
		else:
			instance.queue_free()
	
	loading_finished.emit()
	return true

## Loads the given data to the given instance.
func load_instance(instance: Node, data: Dictionary):
	print("load_instance() called for instance: ", instance.name)
	for property in data.keys():
		if property == "class_name" or property == "name":
			continue
		var value = data[property]
		print("Setting property ", property, " to value ", value)
		
		# First, try setting the property directly
		if instance.has_method("set_" + property):
			instance.call("set_" + property, str_to_var(value))
		elif instance.has_node(NodePath(property)):
			instance.set_indexed(NodePath(property), str_to_var(value))
		else:
			# Check if the property is nested
			if property.find(":") != -1:
				var keys = property.split(":")
				if keys.size() > 1:
					var nested_dict = data
					for i in range(keys.size() - 1):
						if nested_dict.has(keys[i]):
							nested_dict = nested_dict[keys[i]]
						else:
							nested_dict = null
							break
					if nested_dict != null and nested_dict.has(keys[keys.size() - 1]):
						value = nested_dict[keys[keys.size() - 1]]
						property = keys.join("_")
						instance.set(property, str_to_var(value))
						continue
			
			# Finally, try to set it as a property directly
			if instance.has_meta(property):
				instance.set_meta(property, str_to_var(value))
			else:
				if instance.get(property) != null:
					instance.set(property, str_to_var(value))
				else:
					printerr("Property, method, or node path " + str(property) + " does not exist in instance " + instance.name)

## Returns the error message from the last load attempt.
func get_load_error() -> String:
	return load_error

## Returns true if a save file exists.
func save_exists(file_name: String) -> bool:
	return FileAccess.file_exists(get_save_name(file_name))

## Attempts to save the game.
func save_game(file_name := save_name) -> bool:
	save_game_call_count += 1
	if save_game_call_count % 10 == 0:
		print("save_game() called with file_name: ", file_name)
	save_name = file_name
	saving_started.emit()

	var instances = get_tree().get_nodes_in_group("saveable")
	var file = FileAccess.open(get_save_name(), FileAccess.WRITE)
	if file == null:
		save_error = "Failed to create save file. Error: " + str(FileAccess.get_open_error())
		if save_game_call_count % 10 == 0:
			print("save_game: ", save_error)
		saving_failed.emit()
		return false

	var info = {}
	for instance in instances:
		info[instance.name] = collect_instance_properties(instance)

	info["version"] = SaveVersioning.version
	var save_data = JSON.stringify(info)
	if save_data.is_empty():
		save_error = "Failed to convert save data to JSON."
		if save_game_call_count % 10 == 0:
			print("save_game: ", save_error)
		saving_failed.emit()
		return false

	## Create a backup before saving
	if save_exists(save_name):
		var backup_path = save_path + "backup_" + save_name + save_extension
		var backup_file = FileAccess.open(backup_path, FileAccess.WRITE)
		if backup_file:
			var original_file = FileAccess.open(get_save_name(), FileAccess.READ)
			if original_file:
				backup_file.store_string(original_file.get_as_text())
				original_file.close()
			backup_file.close()

	file.store_string(save_data)
	file.close()

	## Calculate and store the hash of the save file for data integrity check
	var file_hash = hash_data(save_data)
	var hash_file = FileAccess.open(get_hash_file_name(), FileAccess.WRITE)
	hash_file.store_string(file_hash)
	hash_file.close()

	saving_finished.emit()
	return true

## Joins the save path, save name, and save extension to return the save's full path.
func get_save_name(file_name := save_name) -> String:
	return save_path + file_name + save_extension

## Joins the save path, save name, and save extension to return the hash file's full path.
func get_hash_file_name(file_name := save_name) -> String:
	return save_path + file_name + ".hash"

## Saves the given instances to the save file.
func collect_instance_properties(instance: Node) -> Dictionary:
	print("collect_instance_properties() called for instance: ", instance.name)
	if instance.get("instance_save_list") == null:
		printerr("Instance " + instance.name + " does not have an instance_save_list variable.")
		return {}
	var properties: Array[String] = instance.instance_save_list.duplicate()
	properties.append("class_name")
	properties.append("name")

	var data: Dictionary = {}
	for property in properties:
		if property is String:
			data[property] = var_to_str(instance.get_indexed(NodePath(property)))
			if data[property] == null:
				printerr("Instance property " + property + " doesn't exist or is null (but it was still saved).")
	return data

## Deletes the save file.
func delete_save(file_name := save_name) -> bool:
	print("delete_save() called with file_name: ", file_name)
	if not save_exists(file_name):
		print("delete_save: No save file exists to delete")
		return false
	var result = DirAccess.remove_absolute(get_save_name()) == OK
	print("delete_save result: ", result)
	return result

## Returns the error message from the last save attempt.
func get_save_error() -> String:
	return save_error

## Handles load failure by providing feedback and recovery options.
func handle_load_failure():
	show_error_message("Failed to load the game. Please try again later or restore from a backup.")
	continue_game()

## Handles save failure by providing feedback and continuing the game.
func handle_save_failure():
	show_error_message("Failed to save the game. Please try again later.")
	continue_game()

## Displays an error message to the player.
func show_error_message(message: String):
	print("Error: ", message)

## Continues the game despite the error.
func continue_game():
	print("Continuing the game despite the error.")

## Calculates a hash for the given data for integrity checks.
func hash_data(data: String) -> String:
	var hasher = HashingContext.new()
	hasher.start(HashingContext.HASH_SHA256)
	hasher.update(data.to_utf8_buffer())
	return hasher.finish().hex_encode()

## Verifies the integrity of the saved data.
func verify_data_integrity(file_name := save_name) -> bool:
	print("verify_data_integrity() called with file_name: ", file_name)
	var save_full_path = get_save_name(file_name)
	print("Save full path: ", save_full_path)
	if not FileAccess.file_exists(save_full_path):
		load_error = "No save file exists."
		print("verify_data_integrity: ", load_error)
		return false

	var file = FileAccess.open(save_full_path, FileAccess.READ)
	if file == null:
		load_error = "Failed to open save file."
		print("verify_data_integrity: ", load_error)
		return false

	var content = file.get_as_text()
	file.close()

	if content.is_empty():
		load_error = "Empty save file."
		print("verify_data_integrity: ", load_error)
		return false

	print("Save file content during verification: ", content)

	var json = JSON.new()
	var parse_error = json.parse(content)
	print("JSON parse result type during verification: ", typeof(parse_error))
	print("JSON parse result during verification: ", parse_error)

	if parse_error != OK:
		load_error = "Failed to parse save file during verification, error code: " + str(parse_error)
		print("verify_data_integrity: ", load_error)
		return false

	var data = json.get_data()
	if typeof(data) != TYPE_DICTIONARY:
		load_error = "Parsed JSON successfully during verification, but result is not a dictionary"
		print("verify_data_integrity: ", load_error)
		return false

	print("Data after parsing during verification: ", data)

	if data.size() == 0:
		load_error = "Parsed data is empty during verification."
		print("verify_data_integrity: ", load_error)
		return false

	var hash_file = FileAccess.open(get_hash_file_name(file_name), FileAccess.READ)
	if hash_file == null:
		load_error = "Failed to open hash file."
		print("verify_data_integrity: ", load_error)
		return false

	var stored_hash = hash_file.get_as_text()
	hash_file.close()

	var current_hash = hash_data(content)
	if stored_hash != current_hash:
		load_error = "Hash mismatch. Data may be corrupted."
		print("verify_data_integrity: ", load_error)
		return false

	return true
