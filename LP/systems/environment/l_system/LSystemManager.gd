extends Node2D

class_name LSystemManager

@export var axiom: String = "F"
@export var rules: Dictionary = {"F": "F[+F]F[-F]F"}
@export var angle: float = 25.0
@export var length: float = 10.0
@export var iterations: int = 4

var l_system: LSystem

# Called when the node enters the scene tree for the first time.
func _ready():
	_generate_l_system()

# Generate L-System based on current properties
func _generate_l_system():
	l_system = LSystem.new(axiom, rules, angle, length, iterations)
	emit_signal("l_system_changed", l_system)
	print("Generated L-System string length:", l_system.generate().length())

# Define signal to notify renderer of changes
signal l_system_changed(l_system)

# Setters to regenerate L-System when properties change
func set_axiom(value: String) -> void:
	axiom = value
	_generate_l_system()

func set_rules(value: Dictionary) -> void:
	rules = value
	_generate_l_system()

func set_angle(value: float) -> void:
	angle = value
	_generate_l_system()

func set_length(value: float) -> void:
	length = value
	_generate_l_system()

func set_iterations(value: int) -> void:
	iterations = value
	_generate_l_system()

# Add an L-System instance
func add_l_system():
	l_system = LSystem.new(axiom, rules, angle, length, iterations)
	emit_signal("l_system_changed", l_system)
	print("L-System added")

# Modify the L-System instance
func modify_l_system(new_axiom: String, new_rules: Dictionary, new_angle: float, new_length: float, new_iterations: int):
	axiom = new_axiom
	rules = new_rules
	angle = new_angle
	length = new_length
	iterations = new_iterations
	_generate_l_system()
	print("L-System modified")
