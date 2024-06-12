extends Node

@export var water_evaporation_per_tick:float

var air_humidity_per_area:Dictionary

func get_air_humidity(area:Vector2i):
	if air_humidity_per_area.has(area):
		return air_humidity_per_area[area]
	else:
		return 0

func absorb_water():
	for i in get_tree().get_nodes_in_group("water_puddles"):
		for area in i.covered_areas:
			if not air_humidity_per_area.has(area):
				air_humidity_per_area[area]=0
			air_humidity_per_area[area]+=water_evaporation_per_tick
			i.reduce(water_evaporation_per_tick)


func _on_tick_timeout():
	absorb_water()
