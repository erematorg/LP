extends Node
class_name WeatherUtilities

static func get_grid_position(position:Vector2)->Vector2i:
	return Vector2i(floor(position.x/WeatherGlobals.grid_size.x),floor(position.y/WeatherGlobals.grid_size.y))

static func num_distance(a:float,b:float):
	if sign(a)!=sign(b):
		return abs(a)+abs(b)
	else:
		return abs(a-b)
