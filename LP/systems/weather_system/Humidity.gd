extends Node

@export var water_evaporation_per_tick:float
## Default amount of moisture the air can hold, in px squared.
## Gets higher with temperature, exactly this value when temperature is
## Temperature.default_temperature
@export var max_air_humidity:float

## How much more moisture can air hold per higher degree of temperature
@export var max_humidity_change:float

var air_humidity_per_area:Dictionary

@onready var temperature: Temperature = get_node("%Temperature")

func get_air_humidity(area:Vector2i):
	if air_humidity_per_area.has(area):
		return air_humidity_per_area[area]
	else:
		return 0

func get_max_humidity(area:Vector2i):
	return max_air_humidity+(max_humidity_change*(temperature.get_temperature(area)-temperature.default_temperature))

func absorb_water():
	for i in get_tree().get_nodes_in_group("water_puddles"):
		for area in i.covered_areas:
			if not air_humidity_per_area.has(area):
				air_humidity_per_area[area]=0
			if air_humidity_per_area[area]<max_air_humidity:
				air_humidity_per_area[area]+=water_evaporation_per_tick
				i.reduce(water_evaporation_per_tick)

func distribute_humidity():
	pass

func _on_tick_timeout():
	absorb_water()
