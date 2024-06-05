@tool class_name GeneticAttributes extends Resource

@export var reproduction_filter_female: Dictionary
@export var reproduction_filter_male: Dictionary

## Overrides current values if they exist in this resource.
@export var preset: GeneticAttributes = null

## When randomizing values based on this weights, the formula applied is [code](value + (value * weight))[/code].
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

## Represents the physical dimensions / length of the entity, influencing its overall appearance.
@export var size := 1.0

## Signifies the mass in kg of the entity, affecting its interactions with other elements in the ecosystem.
@export var weight := 1.0

## Determines the visual appearance of the entity, offering a wide range of color options.
@export var color: Color

## Defines the lifespan of the entity in days.
@export var span := 1

## Specifies the method of reproduction, such as sexual or asexual reproduction.
@export var reproduction_type: REPRODUCTION_TYPES

## Specifies what role this entity is gonna play in reproduction.
@export var sex: SEX

## Determines the trophic levels of the entity (used for predation)
@export var trophic_level := 1


func _init(_preset: GeneticAttributes = null) -> void:
	default_random_weights_preset = true
	default_filter_female_preset = true
	default_filter_male_preset = true

	if is_instance_valid(_preset): preset = _preset
	if is_instance_valid(preset):  apply_preset()


func generate_default_filters() -> Dictionary:
	var result := {}
	for property: Dictionary in GDReflection.get_exported_properties(self).values():
		result[property.name] = false
	return result


func generate_default_random_weights() -> void:
	var result := {}

	for property: Dictionary in GDReflection.get_exported_properties(self).values():
		if property.type in [TYPE_INT, TYPE_FLOAT, TYPE_BOOL]:
			result[property.name] = Vector2(-0.1, 0.1)
		elif property.type == TYPE_COLOR:
			result[property.name] = Gradient.new()
	
	random_weights = result


func apply_preset() -> void:
	GDReflection.transfer_values(self, preset if preset else new())


func set_randomized_values() -> void:
	var properties := GDReflection.get_exported_properties(self).keys()\
		.filter(func(x) -> bool: return !(x in ["preset", "queue_randomize", "queue_apply_preset", "default_filter_female_preset", "default_filter_male_preset", "default_random_weights_preset"]))

	for property in properties:
		if random_weights.has(property):
			var r_weight = random_weights[property]
			var value = get(property)

			if r_weight is Vector2 and (value is float  or value is int or value is bool):
				if value is bool: set(property, bool(randi()))
				else: set(property, value + (value * randf_range(r_weight.x, r_weight.y)))
			
			elif r_weight is Gradient and value is Color:
				if is_instance_valid(r_weight) and !r_weight.offsets.is_empty():
					set(property, r_weight.sample(r_weight.offsets[r_weight.offsets.size() * randf()]))


func merge(with: GeneticAttributes) -> GeneticAttributes:
	var female := with if with.sex == SEX.FEMALE else self
	var male := with if self == female else self
	var merged := GeneticAttributes.new()
	
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
