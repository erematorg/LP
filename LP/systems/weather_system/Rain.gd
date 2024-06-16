extends Node
class_name RainManager

@export var start_raining_on_saturation:float
@export var max_rain_height:float
@export var moisture_loss:float

var columns_raining:Array[int]

@onready var humidity:Humidity=WeatherGlobals.humidity

func _init():
	WeatherGlobals.rain_manager=self

func _process(delta):
	for x in columns_raining:
		var area=Vector2i(x,max_rain_height)
		humidity.saturated_water_per_area[area]-=moisture_loss*delta
		if humidity.saturated_water_per_area[area]<=0:
			humidity.saturated_water_per_area[area]=0
			humidity.saturated_water.emit(area)
			columns_raining.erase(x)

func _on_humidity_saturated_water(area):
	var saturated_water:float=humidity.get_saturated_water(area)
	if saturated_water>=start_raining_on_saturation and not columns_raining.has(area.x):
		columns_raining.append(area.x)

func is_raining_on_area(area)->bool:
	if area.y<=max_rain_height and columns_raining.has(area.x):
		return true
	return false
