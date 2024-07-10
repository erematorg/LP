@tool
extends Resource
class_name Attributes

@export var reproduction_filter_female: Dictionary
@export var reproduction_filter_male: Dictionary
@export var preset: Attributes = null
@export var random_weights := {}
@export var default_filter_female_preset := false:
	set(value): if value and reproduction_filter_female.is_empty(): reproduction_filter_female = generate_default_filters()
@export var default_filter_male_preset := false:
	set(value): if value and reproduction_filter_male.is_empty(): reproduction_filter_male = generate_default_filters()
@export var default_random_weights_preset := false:
	set(value): if value and random_weights.is_empty(): generate_default_random_weights()
@export var queue_apply_preset := false:
	set(value): if value and is_instance_valid(preset): apply_preset()
@export var queue_randomize := false:
	set(value): if value: set_randomized_values()

enum REPRODUCTION_TYPES {
	SEXUAL,
	ASEXUAL
}

enum SEX {
	MALE,
	FEMALE,
	OTHER,
}

@export var size := 1.0
@export var weight := 1.0
@export var color: Color
@export var span := 1
@export var reproduction_type: REPRODUCTION_TYPES
@export var sex: SEX
@export var trophic_level := 1

# Initialize the resource
func _init(_preset: Attributes = null) -> void:
	default_random_weights_preset = true
	default_filter_female_preset = true
	default_filter_male_preset = true
	if is_instance_valid(_preset): preset = _preset
	if is_instance_valid(preset): apply_preset()

# Generate default filters for reproduction
func generate_default_filters() -> Dictionary:
	var result := {}
	for property: Dictionary in GDReflection.get_exported_properties(self).values():
		result[property.name] = false
	return result

# Generate default random weights for attributes
func generate_default_random_weights() -> void:
	var result := {}
	for property: Dictionary in GDReflection.get_exported_properties(self).values():
		if property.type in [TYPE_INT, TYPE_FLOAT, TYPE_BOOL]:
			result[property.name] = Vector2(-0.1, 0.1)
		elif property.type == TYPE_COLOR:
			result[property.name] = Gradient.new()
	random_weights = result

# Apply preset values to the resource
func apply_preset() -> void:
	GDReflection.transfer_values(self, preset if preset else new())

# Set randomized values for attributes
func set_randomized_values() -> void:
	var properties := GDReflection.get_exported_properties(self).keys()\
		.filter(func(x) -> bool: return !(x in ["preset", "queue_randomize", "queue_apply_preset", "default_filter_female_preset", "default_filter_male_preset", "default_random_weights_preset"]))
	for property in properties:
		if random_weights.has(property):
			var r_weight = random_weights[property]
			var value = get(property)
			if r_weight is Vector2 and (value is float or value is int or value is bool):
				if value is bool: set(property, bool(randi()))
				else: set(property, value + (value * randf_range(r_weight.x, r_weight.y)))
			elif r_weight is Gradient and value is Color:
				if is_instance_valid(r_weight) and !r_weight.offsets.is_empty():
					set(property, r_weight.sample(r_weight.offsets[r_weight.offsets.size() * randf()]))

func merge(with: Attributes) -> Attributes:
	var female := with if with.sex == SEX.FEMALE else self
	var male := with if self == female else self
	var merged := Attributes.new()
	merged.apply_preset()
	GDReflection.transfer_values(merged, female, dict_to_array_only_true(female.reproduction_filter_female))
	GDReflection.transfer_values(merged, male, dict_to_array_only_true(male.reproduction_filter_male))
	merged.set_randomized_values()
	return merged

func dict_to_array_only_true(dict: Dictionary) -> Array[String]:
	var result: Array[String] = []
	for key in dict.keys():
		if key is String and dict[key]:
			result.append(key)
	return result
