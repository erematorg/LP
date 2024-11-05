extends Node2D

class_name LSystemManager

@export var axiom: String = "F"
@export var rules: Dictionary = {"F": "F[+F]F[-F]F"}
@export var root_axiom: String = "R"
@export var root_rules: Dictionary = {"R": "R[-R]R[+R]R"}
@export var angle: float = 25.0
@export var length: float = 10.0
@export var iterations: int = 4
@export var root_iterations: int = 3
@export var renderer_path: NodePath = "" # Exported NodePath for the renderer

# Expanded Lifecycle Stages for better flexibility
@export var lifecycle_stages: Array = [
	{ "name": "Seedling", "color": Color(0, 1, 0), "size": 5, "fruit": false },
	{ "name": "Mature", "color": Color(0, 0.8, 0), "size": 15, "fruit": false },
	{ "name": "Fruiting", "color": Color(0, 0.6, 0), "size": 15, "fruit": true, "fruit_color": Color(1, 0, 0) },
	{ "name": "Withering", "color": Color(0.5, 0.25, 0), "size": 10, "fruit": false }
]

@export var timer_interval: float = 5.0  # Interval for lifecycle transitions
@export var branch_density_variation: float = 0.5
@export var leaf_chance: float = 0.3

# Internal variables
var l_system: LSystem
var root_system: LSystem
var current_stage_index: int = 0
var cached_renderer: Node = null

@onready var timer: Timer

# Signal emitted when the L-System is updated
signal l_system_changed(l_system: LSystem, root_system: LSystem)

# Signal emitted when a lifecycle stage changes
signal lifecycle_stage_changed(stage_index: int, stage_data: Dictionary)

# Signal emitted when a segment is interacted with
signal segment_interacted(segment_index: int)

func _ready() -> void:
	timer = Timer.new()
	add_child(timer)
	_initialize_system()

# Initialization function: generate the L-System and set up the timer for lifecycle management
func _initialize_system() -> void:
	_generate_l_system()
	_setup_timer()
	cache_renderer()
	_initialize_lifecycle()

# Cache the renderer node to avoid repeated lookups
func cache_renderer() -> void:
	if !renderer_path.is_empty():
		cached_renderer = get_node(renderer_path)
		if not is_instance_valid(cached_renderer):
			cached_renderer = null
			print("Error: Renderer node is invalid or not found at path:", renderer_path)
	else:
		cached_renderer = null
		print("Warning: Renderer path is empty, renderer not cached.")

# Timer setup with adjustable intervals
func _setup_timer() -> void:
	timer.connect("timeout", Callable(self, "_on_stage_transition"))
	timer.start(timer_interval)

# Generate the L-System with the current parameters and emit the signal
func _generate_l_system() -> void:
	l_system = _create_l_system(axiom, rules, angle, length, iterations)
	root_system = _create_l_system(root_axiom, root_rules, angle, length, root_iterations)
	l_system.branch_density_variation = branch_density_variation
	l_system.leaf_chance = leaf_chance
	emit_signal("l_system_changed", l_system, root_system)
	print("Generated L-System with length:", l_system.generate().length(), "and root length:", root_system.generate().length())

# Helper function to create L-System instances
func _create_l_system(new_axiom: String, new_rules: Dictionary, new_angle: float, new_length: float, new_iterations: int) -> LSystem:
	var l_system_instance = LSystem.new(new_axiom, new_rules, new_angle, new_length, new_iterations)
	return l_system_instance

# Handles the transition between lifecycle stages
func _on_stage_transition() -> void:
	advance_stage()

# Advance to the next lifecycle stage and notify the renderer
func advance_stage() -> void:
	current_stage_index = (current_stage_index + 1) % lifecycle_stages.size()
	var stage = lifecycle_stages[current_stage_index]

	if is_instance_valid(cached_renderer):
		cached_renderer.change_stage(current_stage_index)
		print("Transitioned to stage:", stage["name"], "with parameters:", stage)
	else:
		print("Renderer node missing or invalid")

	emit_signal("lifecycle_stage_changed", current_stage_index, stage)

# Adds a new L-System instance and emits a signal
func add_l_system() -> void:
	_generate_l_system()
	print("New L-System instance added")

# Modifies the L-System parameters dynamically and regenerates the system
func modify_l_system(new_values: Dictionary) -> void:
	var modified = false

	if new_values.has("axiom") and new_values["axiom"] != axiom:
		axiom = new_values["axiom"]
		modified = true
	if new_values.has("rules") and new_values["rules"] != rules:
		rules = new_values["rules"]
		modified = true
	if new_values.has("angle") and new_values["angle"] != angle:
		angle = new_values["angle"]
		modified = true
	if new_values.has("length") and new_values["length"] != length:
		length = new_values["length"]
		modified = true
	if new_values.has("iterations") and new_values["iterations"] != iterations:
		iterations = new_values["iterations"]
		modified = true
	if new_values.has("lifecycle_stages") and new_values["lifecycle_stages"].size() > 0 and new_values["lifecycle_stages"] != lifecycle_stages:
		lifecycle_stages = new_values["lifecycle_stages"]
		modified = true
	if new_values.has("root_axiom") and new_values["root_axiom"] != root_axiom:
		root_axiom = new_values["root_axiom"]
		modified = true
	if new_values.has("root_rules") and new_values["root_rules"] != root_rules:
		root_rules = new_values["root_rules"]
		modified = true
	if new_values.has("root_iterations") and new_values["root_iterations"] != root_iterations:
		root_iterations = new_values["root_iterations"]
		modified = true
	if new_values.has("branch_density_variation") and new_values["branch_density_variation"] != branch_density_variation:
		branch_density_variation = new_values["branch_density_variation"]
		modified = true
	if new_values.has("leaf_chance") and new_values["leaf_chance"] != leaf_chance:
		leaf_chance = new_values["leaf_chance"]
		modified = true

	if modified:
		_generate_l_system()
		print("L-System parameters modified")

# Timer interval setter with validation
func set_timer_interval(value: float) -> void:
	if value > 0:
		timer_interval = value
		timer.stop()
		timer.start(timer_interval)
		print("Timer interval updated to:", timer_interval)
	else:
		print("Invalid timer interval")

# Optional: method to restart lifecycle stages
func reset_lifecycle() -> void:
	current_stage_index = 0
	timer.stop()
	timer.start(timer_interval)
	print("Lifecycle reset to initial stage: Seedling")

# Interact with a specific segment of the L-System
func interact_with_segment(segment_index: int) -> void:
	if segment_index >= 0 and segment_index < l_system.generate().length():
		emit_signal("segment_interacted", segment_index)
		print("Interacted with segment:", segment_index)
	else:
		print("Invalid segment index")

# Initialize lifecycle stages (placeholder for now)
func _initialize_lifecycle() -> void:
	if lifecycle_stages.size() > 0:
		print("Initialized lifecycle stages. Starting with stage:", lifecycle_stages[0]["name"])
	else:
		print("Warning: No lifecycle stages defined.")
