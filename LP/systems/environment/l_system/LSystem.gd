extends Node

class_name LSystem

# L-System parameters
@export var axiom: String
@export var rules: Dictionary
@export var angle: float
@export var length: float
@export var iterations: int
var cached_string: String = ""  # Cache the generated string

# Initialize the L-System with parameters and generate the string
func _init(_axiom: String, _rules: Dictionary, _angle: float, _length: float, _iterations: int):
	axiom = _axiom
	rules = _rules
	angle = _angle
	length = _length
	iterations = _iterations
	_generate_l_system_string()
	print("LSystem initialized")

# Generate the L-System string and cache it, adding slight randomness to angle and length
func _generate_l_system_string() -> void:
	var current = axiom
	for i in range(iterations):
		var next = ""
		for c in current:
			if rules.has(c):
				next += rules[c]
			else:
				next += c
		current = next
	cached_string = current  # Cache the generated string

# Public method to get the cached L-System string
func generate() -> String:
	if cached_string == "":
		_generate_l_system_string()
	return cached_string

# Methods for setting and getting parameters (regenerate string if needed)
func set_axiom(value: String) -> void:
	axiom = value
	_generate_l_system_string()

func get_axiom() -> String:
	return axiom

func set_rules(value: Dictionary) -> void:
	rules = value
	_generate_l_system_string()

func get_rules() -> Dictionary:
	return rules

func set_angle(value: float) -> void:
	angle = value

func get_angle() -> float:
	return angle

func set_length(value: float) -> void:
	length = value

func get_length() -> float:
	return length

func set_iterations(value: int) -> void:
	iterations = value
	_generate_l_system_string()

func get_iterations() -> int:
	return iterations
