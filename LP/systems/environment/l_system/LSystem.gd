extends Node

class_name LSystem

# L-System parameters for branches
@export var axiom: String = "F"
@export var rules: Dictionary = {"F": "FF+[+F-F-F]-[-F+F+F]"}
@export var angle: float = 25.0
@export var length: float = 10.0
@export var iterations: int = 3

# Parameters for roots (separate if needed)
@export var root_angle: float = 45.0
@export var root_length: float = 7.0

# Randomness for procedural generation
@export var angle_variation: float = 5.0
@export var length_variation: float = 0.1

# Cached data
var cached_string: String = ""
var cached_angles: Array = []
var cached_lengths: Array = []

# Initialize and generate L-System
func _init(_axiom: String, _rules: Dictionary, _angle: float, _length: float, _iterations: int) -> void:
	axiom = _axiom
	rules = _rules
	angle = _angle
	length = _length
	iterations = _iterations
	_generate_l_system_string()

# Generate L-System string with randomness
func _generate_l_system_string() -> void:
	var current = axiom
	cached_angles.clear()
	cached_lengths.clear()

	for i in range(iterations):
		var next = ""
		for char in current:
			next += rules.get(char, char)
		current = next

	cached_string = current

	for char in cached_string:
		if char == "F":
			cached_angles.append(_randomize_angle(angle))
			cached_lengths.append(_randomize_length(length))

# Helper function for randomized angle
func _randomize_angle(base_angle: float) -> float:
	return deg_to_rad(base_angle + randf_range(-angle_variation, angle_variation))

# Helper function for randomized length
func _randomize_length(base_length: float) -> float:
	return base_length * randf_range(1.0 - length_variation, 1.0 + length_variation)

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
