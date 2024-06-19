extends Node
## Transmits Humidity according to the wind.

@onready var wind_manager:=WeatherGlobals.wind

func _on_tick_timeout():
	if not WeatherGlobals.tick.total_ticks%10==0:
		return
	for area in get_parent().saturated_water_per_area.keys():
		var wind=wind_manager.get_wind_on_area(area)
		var new_area:Vector2i
		if wind.x<0:
			new_area=area+Vector2i(-1,0)
		if wind.x>0:
			new_area=area+Vector2i(1,0)
		var to_transfer=get_parent().decrease_humidity(area,abs(wind.x)/10)
		get_parent().add_to_humidity(new_area,to_transfer)
