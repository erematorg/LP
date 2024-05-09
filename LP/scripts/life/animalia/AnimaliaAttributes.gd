@tool class_name AnimaliaAttributes extends GeneticAttributes

enum FOOD_TYPES {
	CARNIVOROUS,
	HERBIVOROUS,
}

## Should use FOOD_TYPES but gdscript
@export_flags("CARNIVOROUS","HERBIVOROUS") var food_types: int

@export var leg_count := 1
@export var has_wings := false
@export var movement_type := 1
