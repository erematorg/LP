class_name SaveVersioning
extends Resource

static var version = "1.1.0"
static var version_history = ["1.0.0", "1.1.0"]
var loaded_data: Dictionary = {}

func _init():
	assert(SaveVersioning.validate_system_integrity(), "SaveVersioning integrity check failed.")

static func validate_system_integrity() -> bool:
	for i in range(1, version_history.size()):
		if version_history[i] < version_history[i - 1]:
			printerr("Version history is not in order.")
			return false
	return true

static func is_data_up_to_date(data: Dictionary) -> bool:
	if !data.has("version"):
		printerr("Save file does not have a version number.")
		return false
	return data["version"] == version

func update_data(data: Dictionary) -> Dictionary:
	loaded_data = data
	var index = version_history.find(data["version"]) + 1

	while index < version_history.size():
		var method_name = "upgrade_" + version_history[index].replace(".", "_")
		if has_method(method_name):
			call(method_name)
			print("Conversion to ", version_history[index], " complete.")
		else:
			printerr("Conversion method '" + method_name + "' not found. Skipping conversion.")
		index += 1

	loaded_data["version"] = version
	return loaded_data

## Version converters
func upgrade_1_0_0():
	pass

func upgrade_1_1_0():
	rename_property("player_info", "player_data")
	change_property_type("player_data", "This comes from 1.0.0, but it was converted to 1.1.0!")

## Conversion Rules
func rename_property(from: String, to: String):
	for instance in loaded_data.keys():
		if instance == "version": continue
		if loaded_data[instance].has(from):
			print("Renaming property '" + from + "' to '" + to + "'.")
			loaded_data[instance][to] = loaded_data[instance][from]
			loaded_data[instance].erase(from)

func delete_property(property: String):
	for instance in loaded_data.keys():
		if instance == "version": continue
		if loaded_data[instance].has(property):
			print("Deleting property '" + property + "'.")
			loaded_data[instance].erase(property)

func change_property_type(property: String, value):
	for instance in loaded_data.keys():
		if instance == "version": continue
		if loaded_data[instance].has(property):
			print("Changing type of property '" + property + "' to '" + value + "'.")
			loaded_data[instance][property] = value

func change_instance_name(from: String, to: String):
	for instance in loaded_data.keys():
		if instance == "version": continue
		if instance == from:
			print("Changing instance name '" + from + "' to '" + to + "'.")
			loaded_data[to] = loaded_data[instance]
			loaded_data.erase(instance)
