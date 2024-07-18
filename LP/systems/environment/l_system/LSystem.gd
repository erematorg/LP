extends Node

class_name LSystem

# L-System parameters
@export var axiom: String
@export var rules: Dictionary
@export var angle: float
@export var length: float
@export var iterations: int

# Initialize the L-System with parameters
func _init(_axiom: String, _rules: Dictionary, _angle: float, _length: float, _iterations: int):
	axiom = _axiom
	rules = _rules
	angle = _angle
	length = _length
	iterations = _iterations
	print("LSystem initialized")

# Generate the L-System string based on the parameters
func generate() -> String:
	var current = axiom
	for i in range(iterations):
		var next = ""
		for c in current:
			if rules.has(c):
				next += rules[c]
			else:
				next += c
		current = next
	return current

# Methods for setting and getting parameters
func set_axiom(value: String) -> void:
	axiom = value

func get_axiom() -> String:
	return axiom

func set_rules(value: Dictionary) -> void:
	rules = value

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

func get_iterations() -> int:
	return iterations
