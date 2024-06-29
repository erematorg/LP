extends WeatherFallingEffect

@onready var visibility:=WeatherGlobals.area_visibility

func _ready():
	super._ready()

func _is_area_needed(area:Vector2i):
	return WeatherGlobals.rain_manager.is_raining_on_area(area) and WeatherGlobals.temperature.get_temperature(area)<0
