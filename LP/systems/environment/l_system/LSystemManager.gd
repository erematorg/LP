extends Node2D

class_name LSystemManager

@export var axiom: String = "F"
@export var rules: Dictionary = {"F": "F[+F]F[-F]F"}
@export var angle: float = 25.0
@export var length: float = 10.0
@export var iterations: int = 4
@export var lifecycle_stages: Array = [
	{ "name": "Seedling", "color": Color(0, 1, 0), "size": 5 },
	{ "name": "Mature", "color": Color(0, 1, 0), "size": 10 },
	{ "name": "Fruiting", "color": Color(0, 1, 0), "size": 10, "fruit_color": Color(1, 0, 0) },
	{ "name": "Withering", "color": Color(0.5, 0.25, 0), "size": 5 }
]

var l_system: LSystem
var current_stage_index: int = 0

# Timer for stage transitions
@onready var timer: Timer = Timer.new()

signal l_system_changed(l_system)

# Called when the node enters the scene tree for the first time
func _ready():
	_generate_l_system()
	add_child(timer)
	timer.connect("timeout", Callable(self, "_on_stage_transition"))
	timer.start(5)  # Transition stages every 5 seconds

# Generate L-System based on current properties
func _generate_l_system():
	l_system = LSystem.new(axiom, rules, angle, length, iterations)
	emit_signal("l_system_changed", l_system)
	print("Generated L-System string length:", l_system.generate().length())

# Add an L-System instance
func add_l_system():
	l_system = LSystem.new(axiom, rules, angle, length, iterations)
	emit_signal("l_system_changed", l_system)
	print("L-System added")

# Modify the L-System instance
func modify_l_system(new_axiom: String, new_rules: Dictionary, new_angle: float, new_length: float, new_iterations: int, new_lifecycle_stages: Array):
	axiom = new_axiom
	rules = new_rules
	angle = new_angle
	length = new_length
	iterations = new_iterations
	lifecycle_stages = new_lifecycle_stages
	_generate_l_system()
	print("L-System modified")

# Handle stage transitions
func _on_stage_transition():
	current_stage_index = (current_stage_index + 1) % lifecycle_stages.size()
	var renderer = get_parent().get_node("LSystemRenderer")
	if is_instance_valid(renderer):
		renderer.change_stage(current_stage_index)
		print("Transitioned to stage:", lifecycle_stages[current_stage_index].name)
	else:
		print("Renderer is invalid or missing")
	emit_signal("l_system_changed", l_system)

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

# Set lifecycle stages and regenerate the L-System with validation
func set_lifecycle_stages(value: Array) -> void:
	if value.size() > 0:
		lifecycle_stages = value
		_generate_l_system()
	else:
		print("Invalid lifecycle stages provided.")

# Get the current lifecycle stages
func get_lifecycle_stages() -> Array:
	return lifecycle_stages
