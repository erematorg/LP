extends Node
class_name RainManager

@export var start_raining_on_saturation:float
@export var max_rain_height:int
@export var moisture_loss:float

# Only holds those areas in which it's raining, and the values are the water left to rain.
var columns_raining:Dictionary

@onready var humidity:Humidity=WeatherGlobals.humidity

func _init():
	WeatherGlobals.rain_manager=self


func _on_humidity_saturated_water(area):
	var saturated_water:float=humidity.get_saturated_water(area)
	if saturated_water>=start_raining_on_saturation and not columns_raining.has(area.x):
		columns_raining[area.x]=saturated_water

func is_raining_on_area(area)->bool:
	if area.y>=max_rain_height and columns_raining.has(area.x):
		return true
	return false


func _on_tick_timeout():
	for x in columns_raining.keys():
		var area=Vector2i(x,max_rain_height)
		humidity.decrease_humidity(area,moisture_loss)
		columns_raining[x]-=moisture_loss
		if humidity.saturated_water_per_area[area]<=0:
			humidity.saturated_water_per_area[area]=0
			columns_raining.erase(x)
		elif columns_raining[x]<=0:
			columns_raining.erase(x)
		
		humidity.saturated_water.emit(area)
