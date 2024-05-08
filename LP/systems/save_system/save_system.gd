class_name SaveSystem
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
var save_extention := ".sav"

var autosave_interval := -1.0
var time := 0.0

## Stores the data loaded from the save file.
var loaded_data: Dictionary = {}

func _ready():
	# By default, the process function is disabled.
	set_process(false)

	setup_autosave(0.1)

	if save_exists(save_name):
		load_game()
	
	save_game()


## Called every frame.
func _process(delta):
	time += delta

	if time >= autosave_interval:
		save_game()
		time = 0.0

## Sets up the autosave timer. If the interval is less than or equal to 0, the autosave timer is disabled.
## It will always save using the last save file name.
func setup_autosave(interval: float):
	if interval <= 0:
		set_process(false)
		return
	
	set_process(true)
	
	autosave_interval = interval
	time = 0.0

## Attempts to load the game. Returns true if successful, false otherwise.
func load_game(file_name := save_name) -> bool:
	# Sets the save name.
	save_name = file_name

	loading_started.emit()

	if !save_exists(save_name):
		load_error = "No save file exists."
		loading_failed.emit()
		return false
	
	# Opens the save file and parses it.
	var file := FileAccess.open(get_save_name(), FileAccess.READ)
	
	var json := JSON.new()
	
	var content = file.get_as_text()
	var data: Dictionary = JSON.parse_string(content)
	
	file.close()

	if !SaveVersioning.is_data_up_to_date(data):
		# If the data is not up to date, updates it.
		print("Save data is not up to date. Updating...")

		var data_conversor := SaveVersioning.new()
		data = data_conversor.update_data(data)
	
	# Checks if the save file was parsed successfully.
	if data == null:
		load_error = "Failed to parse save file. Error: " + str(json.get_error_message())
		loading_failed.emit()
		return false

	# Gets saveable instances.
	for instance in get_tree().get_nodes_in_group("saveable"):
		# If found in the save file, loads the data. Else, deletes the instance.
		if data.keys().has(instance.name):
			var instance_data: Dictionary = data[instance.name]

			# Sets the data to the instance.
			load_instance(instance, instance_data)
		else:
			instance.queue_free()
	
	# Finallizes the loading process.
	loading_finished.emit()

	return true


## Loads the given data to the given instance.
func load_instance(instance: Node, data: Dictionary):
	for property in data.keys():
		if property == "class_name" or property == "name": continue

		var nodepath := NodePath(property)
		var value = data[property]

		instance.set_indexed(nodepath, str_to_var(value))

## Returns the error message from the last load attempt.
func get_load_error() -> String:
	return load_error

## Returns true if a save file exists, false otherwise.
func save_exists(file_name: String) -> bool:
	return FileAccess.file_exists(get_save_name(file_name))


## Attemps to save the game. Returns true if successful, false otherwise.
func save_game(file_name := save_name) -> bool:
	# Sets the save name.
	save_name = file_name
	
	saving_started.emit()

	# Collects all instances that should be saved.
	var instances := get_tree().get_nodes_in_group("saveable")

	# Creates the save file.
	var file := FileAccess.open(get_save_name(), FileAccess.WRITE)
	# Checks if the file was created successfully.
	if file == null:
		save_error = "Failed to create save file. Error: " + str(FileAccess.get_open_error())
		saving_failed.emit()
		return false
	
	# Collects info of the instances
	var info := {}
	for instance in instances:
		info.merge({instance.name: collect_instance_properties(instance)})
	
	info["version"] = SaveVersioning.version
	
	# Saves the data to the file.
	file.store_string(JSON.stringify(info))
			
	# Closes the file.
	file.close()

	saving_finished.emit()

	return true

## Joins the save path, save name and save extention to return the save's full path.
func get_save_name(file_name := save_name) -> String:
	return save_path + file_name + save_extention

## Saves the given instances to the save file.
func collect_instance_properties(instance: Node) -> Dictionary:
	if instance.get("instance_save_list") == null:
		printerr("Instance " + instance.name + " does not have an instance_save_list variable.")
		return {}
	
	var properties: Array[String] = instance.instance_save_list.duplicate()
	
	properties.append("class_name")
	properties.append("name")

	var data: Dictionary = {}
	
	# Gets the data from the instance's instance_save_list variable.
	for property in properties:
		if property is String:
			data[property] = var_to_str(instance.get_indexed(NodePath(property)))
			
			if data[property] == null:
				printerr("Instance property " + property + " doesn't exist or is null (but it was still saved).")
	
	return data
	
## Deletes the save file. Returns true if successful, false otherwise.
func delete_save(file_name := save_name) -> bool:
	if !save_exists(file_name):
		return false
	
	return true if (DirAccess.remove_absolute(get_save_name()) == OK) else false

## Returns the error message from the last save attempt.
func get_save_error() -> String:
	return save_error


