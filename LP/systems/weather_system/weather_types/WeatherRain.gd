extends WeatherFallingEffect
class_name WeatherRain

@export var drops_alpha_curve : CurveTexture
@export var debug_hints_enabled:bool = false

func _ready():
	super._ready()



func _is_area_needed(area:Vector2i):
	return WeatherGlobals.rain_manager.is_raining_on_area(area) and WeatherGlobals.temperature.get_temperature(area)>=0
