class_name SaveVersioning
extends Resource

## The current version of the save file system.
static var version = "1.0.0"
## The history of versions of the save file system.
static var version_history = ["1.0.0", "1.1.0"]

## The data that is loaded from the save file. Used internally, shouldn't be used by other scripts.
var loaded_data: Dictionary = {}
	
func _init():
	# Error checking
	assert(SaveVersioning.validate_system_integrity(), "SaveVersioning integrity check failed. See output for more details.")

## Checks if the save file system is in a valid state. Returns true if it is, false otherwise.
static func validate_system_integrity() -> bool:
	# Checks if the version history is in order.
	for i in range(1, version_history.size()):
		if version_history[i] < version_history[i - 1]:
			printerr("Version history is not in order.")
			return false
	
	return true

## Checks if the given save data is up to date.
static func is_data_up_to_date(data: Dictionary) -> bool:
	if !data.has("version"):
		printerr("Save file does not have a version number.")
		return false
	
	return data["version"] == version

## Returns the major version number of the save file system.
static func get_major_version() -> int:
	return int(version.split(".")[0])

## Returns the minor version number of the save file system.
static func get_minor_version() -> int:
	return int(version.split(".")[1])

## Returns the patch version number of the save file system.
static func get_patch_version() -> int:
	return int(version.split(".")[2])

## Updates the save file to the current version.
func update_data(data: Dictionary) -> Dictionary:
	loaded_data = data

	var index = version_history.find(data["version"]) + 1
	
	# Updates the save file to the current version.
	while index < version_history.size():
		var method_name = "upgrade_" + version_history[index].replace(".", "_")
		
		if has_method(method_name):
			# Method found
			call(method_name)
			print("Conversion to ", version_history[index]," complete.")
		else:
			# Method not found
			printerr("Conversion method '" + method_name + "' not found. Skipping conversion.")
		index += 1
	
	# Updates the version number.
	loaded_data["version"] = version
	
	return data
	
# ----- Version conversors -----
# 1.0.0
func upgrade_1_0_0():
	pass

# 1.1.0
func upgrade_1_1_0():
	rename_property("player_info", "player_data")
	change_property_type("player_data", "This comes from 1.0.0, but it was converted to 1.1.0!")


# ----- Conversion Rules -----
## Renames a property in the save file.
func rename_property(from: String, to: String):
	# Loops through all instances in the save file.
	for instance in loaded_data.keys():
		if instance == "version": continue
		
		# Loops through all properties in the instance.
		for property in loaded_data[instance].keys():
			if property == from:
				print("Renaming property '" + from + "' to '" + to + "'.")
				loaded_data[instance][to] = loaded_data[instance][property]
				loaded_data[instance].erase(property)

## Deletes a property from the save file.
func delete_property(property: String):
	# Loops through all instances in the save file.
	for instance in loaded_data.keys():
		if instance == "version": continue
		
		# Loops through all properties in the instance.
		for prop in loaded_data[instance].keys():
			if prop == property:
				print("Deleting property '" + property + "'.")
				loaded_data[instance].erase(property)

## Changes the value of a property in the save file.
func change_property_type(property: String, value):
	# Loops through all instances in the save file.
	for instance in loaded_data.keys():
		if instance == "version": continue
		
		# Loops through all properties in the instance.
		for prop in loaded_data[instance].keys():
			if prop == property:
				print("Changing type of property '" + property + "' to '" + value + "'.")
				loaded_data[instance][property] = var_to_str(value)

## Changes the name of an instance in the save file.
func change_instance_name(from: String, to: String):
	# Loops through all instances in the save file.
	for instance in loaded_data.keys():
		if instance == "version": continue
		
		if instance == from:
			print("Changing instance name '" + from + "' to '" + to + "'.")
			loaded_data[to] = loaded_data[instance]
			loaded_data.erase(instance)
