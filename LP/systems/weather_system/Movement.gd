extends Node
@export var enabled:bool

@onready var wind_manager=WeatherGlobals.wind
@onready var cloud_spawner:CloudSpawner=get_parent().get_parent()

func _process(delta):
	if not enabled:
		return
	
	var wind: float=wind_manager.get_wind_on_area(get_parent().area)
	get_parent().position+=Vector2(wind,0)*(delta/WeatherGlobals.tick.wait_time)
	
	# Check in case if we moved out of the area.
	var new_area:Vector2i = WeatherUtilities.get_grid_position(get_parent().position+WeatherGlobals.grid_size/2)
	if new_area!=get_parent().area:
		cloud_spawner.remove_drawer(get_parent(),get_parent().area)
		cloud_spawner.add_drawer(get_parent(),new_area)
		get_parent().area=new_area
