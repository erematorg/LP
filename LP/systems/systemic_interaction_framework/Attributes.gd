@tool
extends Resource
class_name Attributes

# General attributes shared across all entities
@export var size := 1.0
@export var weight := 1.0
@export var color: Color = Color(1, 1, 1)
@export var span := 1  # Lifespan or size span
@export var trophic_level := 1  # Position in the food chain
@export var reproduction_type: REPRODUCTION_TYPES = REPRODUCTION_TYPES.SEXUAL
@export var sex: SEX = SEX.OTHER  # Default to OTHER unless specified

# Optional: Randomization and presets for attributes
@export var random_weights := {}
@export var preset: Attributes = null
@export var queue_apply_preset := false:
	set(value):
		if value and is_instance_valid(preset):
			apply_preset()

@export var queue_randomize := false:
	set(value):
		if value:
			set_randomized_values()

# Reproduction types
enum REPRODUCTION_TYPES {
	SEXUAL,
	ASEXUAL
}

# Sex definitions
enum SEX {
	MALE,
	FEMALE,
	OTHER
}

# Initialize the resource with optional preset
func _init(_preset: Attributes = null) -> void:
	if is_instance_valid(_preset):
		preset = _preset
	if is_instance_valid(preset):
		apply_preset()

# Apply preset values to the resource
func apply_preset() -> void:
	GDReflection.transfer_values(self, preset if preset else new())
	print("Applied preset values to:", self)  # Debug print

# Set randomized values for attributes based on weights
func set_randomized_values() -> void:
	print("Randomizing values for:", self)  # Debug print
	var properties = GDReflection.get_exported_properties(self).keys().filter(func(x) -> bool:
		return !(x in ["preset", "queue_randomize", "queue_apply_preset"])
	)
	for property in properties:
		if random_weights.has(property):
			var r_weight = random_weights[property]
			var value = get(property)
			if r_weight is Vector2 and (value is float or value is int or value is bool):
				if value is bool:
					set(property, bool(randi() % 2))
				else:
					set(property, value + (value * randf_range(r_weight.x, r_weight.y)))
			elif r_weight is Gradient and value is Color:
				if is_instance_valid(r_weight) and !r_weight.offsets.is_empty():
					set(property, r_weight.sample(r_weight.offsets[r_weight.offsets.size() * randf()]))

# Generate default random weights for randomization
func generate_default_random_weights() -> void:
	var result := {}
	for property in GDReflection.get_exported_properties(self).values():
		if property.type in [TYPE_INT, TYPE_FLOAT, TYPE_BOOL]:
			result[property.name] = Vector2(-0.1, 0.1)  # Default variation range
		elif property.type == TYPE_COLOR:
			result[property.name] = Gradient.new()
	random_weights = result
