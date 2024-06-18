extends Node

@export var enabled:bool
@export var arrow:PackedScene


func _on_tick_timeout():
	if not enabled:
		return
	for i in get_children():
		i.queue_free()
	for area in WeatherGlobals.area_visibility.shown_areas:
		var wind=WeatherGlobals.wind.get_wind_on_area(area)
		var new_arrow=arrow.instantiate()
		new_arrow.position=WeatherUtilities.get_real_position(area)
		new_arrow.position+=WeatherGlobals.grid_size/2
		var arrow=new_arrow.get_node("Arrow")
		if wind.length()!=0:
			arrow.scale=Vector2(wind.length(),1)
		else:
			arrow.color=Color.INDIAN_RED
		arrow.rotation=wind.angle()
		new_arrow.get_node("Amount").rotation=-new_arrow.rotation
		new_arrow.get_node("Amount").text=str(wind)
		add_child(new_arrow)
		
