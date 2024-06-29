extends Node

@export var enabled:bool
@export var indicator_template:PackedScene


func _on_tick_timeout():
	if not enabled:
		return
	for i in get_children():
		i.queue_free()
	for area in WeatherGlobals.area_visibility.shown_areas:
		var wind=WeatherGlobals.wind.get_wind_on_area(area)
		var indicator=indicator_template.instantiate()
		indicator.position=WeatherUtilities.get_real_position(area)
		indicator.position+=WeatherGlobals.grid_size/2
		var arrow=indicator.get_node("Arrow")
		if wind!=0:
			arrow.scale=Vector2(wind,1)
		else:
			arrow.color=Color.INDIAN_RED
		indicator.get_node("Amount").text=str(wind)
		add_child(indicator)
		
