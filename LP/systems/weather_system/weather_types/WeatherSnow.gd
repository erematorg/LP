extends WeatherFallingEffect
class_name WeatherSnow

@onready var visibility = WeatherGlobals.area_visibility

func _ready():
	super._ready()

## Checks if it should snow, and if it's below freezing.
func _is_area_needed(area: Vector2i) -> bool:
	return WeatherGlobals.rain_manager.is_raining_on_area(area) and WeatherGlobals.temperature.get_temperature(area) < 0
