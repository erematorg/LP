extends Node
class_name GrowthComponent

@export var growth_rate: float = 1.0

func initialize(data):
	if data.has("growth_rate"):
		growth_rate = data.growth_rate

func _ready():
	print("GrowthComponent ready with growth rate: ", growth_rate)
