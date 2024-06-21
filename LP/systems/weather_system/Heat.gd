extends Node
class_name Temperature

## Used before taking into account altitude
@export var default_temperature:float

## Higher air is less dense, and as such colder.
## This controls how much base temperature changes per unit of altitude.
@export var altitude_temperature_change:float
var temperature_per_area:Dictionary

func _init():
	WeatherGlobals.temperature=self

func get_temperature(area:Vector2i):
	if not temperature_per_area.has(area):
		temperature_per_area[area]=default_temperature
		temperature_per_area[area]+=altitude_temperature_change*area.y
	return temperature_per_area[area]
