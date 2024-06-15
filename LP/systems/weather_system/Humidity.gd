extends Node
class_name Humidity

@export var saturated_water_per_area: Dictionary

@export var water_evaporation_per_tick:float
## Default amount of moisture the air can hold, in px squared.
## Gets higher with temperature, exactly this value when temperature is
## Temperature.default_temperature
@export var max_air_humidity:float

## How much more moisture can air hold per higher degree of temperature
@export var max_humidity_change:float

## Moisture tends to go to adjacent dryer air,
## using this speed
## It's in px squared per tick
@export var humidity_transfer_speed:float

## Moisture tends to go to adjacent dryer air,
## using this speed when going up. It should be higher than humidity_transfer speed because
## moistured air rises
## It's in px squared per tick
@export var humidity_elevation_speed:float

## Maximum height at which moisture won't be transmitted up.
## Clouds will normally form here. Remember y is inverted.
@export var max_moisture_height:int

var air_humidity_per_area:Dictionary

@onready var temperature: Temperature = get_node("%Temperature")

func get_air_humidity(area:Vector2i):
	if air_humidity_per_area.has(area):
		return air_humidity_per_area[area]
	else:
		return 0


func get_max_humidity(area:Vector2i):
	return max_air_humidity+(max_humidity_change*(temperature.get_temperature(area)-temperature.default_temperature))

func add_to_humidity(of_area:Vector2i,amount:float):
	var 

## From 0 to 1, 1 meaning the air cant hold any more moisture
func get_relative_humidity(area:Vector2i):
	return (get_air_humidity(area)/get_max_humidity(area))

func absorb_water():
	for i in get_tree().get_nodes_in_group("water_puddles"):
		for area in i.covered_areas:
			if not air_humidity_per_area.has(area):
				air_humidity_per_area[area]=0
			if air_humidity_per_area[area]<max_air_humidity:
				air_humidity_per_area[area]+=water_evaporation_per_tick
				i.reduce(water_evaporation_per_tick)

func distribute_humidity():
	for area in air_humidity_per_area.keys():
		var areas_to_distribute_to=[
			area+Vector2i(1,0),
			area+Vector2i(-1,0),
		]
		if area.y>max_moisture_height:
			areas_to_distribute_to.append_array([
			area+Vector2i(1,-1),
			area+Vector2i(-1,-1),
			area+Vector2i(0,-1),
			])
		for new_area in areas_to_distribute_to:
			if not air_humidity_per_area.has(new_area):
				air_humidity_per_area[new_area]=0
			if air_humidity_per_area[new_area]<air_humidity_per_area[area]:
				var transfer_speed:float=humidity_transfer_speed
				if new_area.y<area.y:
					transfer_speed=humidity_elevation_speed
				var amount_to_transfer=clamp(transfer_speed,0,air_humidity_per_area[area])
				air_humidity_per_area[new_area]+=amount_to_transfer
				air_humidity_per_area[area]-=amount_to_transfer

func _on_tick_timeout():
	absorb_water()
	distribute_humidity()
