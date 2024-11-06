extends Resource

class_name LSystem

# L-System parameters for branches
@export var axiom: String = "F"
@export var rules: Dictionary = {
	"F": "FF-[-F+F+F]+[+F-F-F]L",
	"R": "RR-[-R+R]+[+R-R]"
}
@export var angle: float = 25.0
@export var length: float = 10.0
@export var iterations: int = 3

# Parameters for roots
@export var root_angle: float = 45.0
@export var root_length: float = 7.0
@export var root_angle_variation: float = 5.0
@export var root_length_variation: float = 0.1

# Randomness for procedural generation
@export var angle_variation: float = 5.0
@export var length_variation: float = 0.1
@export var branch_density_variation: float = 0.5
@export var leaf_chance: float = 0.3

# Cached data
var cached_string: String = ""
var cached_angles: Array = []
var cached_lengths: Array = []
var cached_symbols: Array = []

# Memoization cache for generated L-System strings
var l_system_cache: Dictionary = {}

# Initialize and generate L-System
func _init(_axiom: String, _rules: Dictionary, _angle: float, _length: float, _iterations: int) -> void:
	axiom = _axiom
	rules = _rules
	angle = _angle
	length = _length
	iterations = _iterations
	if validate_rules():
		_generate_l_system_string()

# Generate L-System string with randomness and memoization
func _generate_l_system_string() -> void:
	var key = str(axiom, rules, iterations)

	if l_system_cache.has(key):
		cached_string = l_system_cache[key]
		return

	var current = axiom
	cached_angles.clear()
	cached_lengths.clear()
	cached_symbols.clear()

	for i in range(iterations):
		var next = ""
		for symbol in current:
			next += rules.get(symbol, symbol)
		current = next

	cached_string = current

	for symbol in cached_string:
		if symbol == "F":
			cached_angles.append(_randomize_branch_angle(angle))
			cached_lengths.append(_randomize_branch_length(length))
			cached_symbols.append("branch")
			# Add leaves at branch tips
			if randf() < leaf_chance:
				cached_symbols.append("leaf")
		elif symbol == "R":  # Handling roots
			cached_angles.append(_randomize_root_angle(root_angle))
			cached_lengths.append(_randomize_root_length(root_length))
			cached_symbols.append("root")

	l_system_cache[key] = cached_string

# Randomize angles for branches
func _randomize_branch_angle(base_angle: float) -> float:
	var noise_factor = randf_range(-angle_variation, angle_variation)
	return deg_to_rad(base_angle + noise_factor)

# Randomize lengths for branches
func _randomize_branch_length(base_length: float) -> float:
	return base_length * randf_range(1.0 - length_variation, 1.0 + length_variation)

# Randomize angles for roots
func _randomize_root_angle(base_angle: float) -> float:
	var noise_factor = randf_range(-root_angle_variation, root_angle_variation)
	return deg_to_rad(base_angle + noise_factor)

# Randomize lengths for roots
func _randomize_root_length(base_length: float) -> float:
	return base_length * randf_range(1.0 - root_length_variation, 1.0 + root_length_variation)

# Validate rules to ensure they are strings
func validate_rules() -> bool:
	for key in rules.keys():
		if typeof(rules[key]) != TYPE_STRING:
			push_error("Rule for '" + str(key) + "' is not a valid string.")
			return false
	return true

# Retrieve generated string
func generate() -> String:
	if cached_string == "":
		_generate_l_system_string()
	return cached_string

# Retrieve cached angles
func get_cached_angles() -> Array:
	return cached_angles

# Retrieve cached lengths
func get_cached_lengths() -> Array:
	return cached_lengths

# Retrieve cached symbols
func get_cached_symbols() -> Array:
	return cached_symbols
