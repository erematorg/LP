extends WeatherFallingEffect
class_name WeatherRain

@export var drops_alpha_curve : CurveTexture
@export var debug_hints_enabled:bool = false

func _ready():
	super._ready()

## Checks if it should rain, and if it's not freezing.
func _is_area_needed(area:Vector2i):
	return WeatherGlobals.rain_manager.is_raining_on_area(area) and WeatherGlobals.temperature.get_temperature(area)>=0
