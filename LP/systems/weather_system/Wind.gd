extends Node
class_name Wind
## Manages wind stats, does not depend on anything but [annotation Tick.total_ticks]

## Distance between points with 0 wind.
@export var neutral_point_distance: float

## Minimal amount of wind when not 0, scaled when an area is further away of a neutral point
## or on higher altitudes. in px/tick
@export var minimal_wind_unit: float

## How much the amount of wind gets affected by the distance to a neutral point.
@export var wind_per_distance_to_neutral_point: float

## For each point of altitude, the wind gets added this percentage. 0.1 is 10%.
@export var added_multiplier_per_altitude: float

@export var max_wind: float

## Amount of ticks in which the wind direction is the same.
@export var ticks_per_cycle: int

## With time, the function scrolls to the right. This is the amount of time it takes
## to move 1 area.
@export var ticks_per_x_change: float

func _init():
	WeatherGlobals.wind = self

func get_wind_on_area(area: Vector2i) -> float:
	area = Vector2(area)
	if WeatherGlobals.tick == null:
		return 0.0
	var x_change = floor(WeatherGlobals.tick.total_ticks / ticks_per_x_change)
	area.x += x_change
	## The step of a transition between neutral points in which the area it's in.
	var neutral_point_index = floor(area.x / neutral_point_distance)
	var neutral_point = neutral_point_index * neutral_point_distance
	var stage = area.x - neutral_point
	var wind = 0.0
	if stage < neutral_point_distance / 2:
		var distance_to_next_neutral_point = neutral_point_distance - stage
		wind = minimal_wind_unit + distance_to_next_neutral_point * wind_per_distance_to_neutral_point
	elif stage >= neutral_point_distance / 2:
		wind = -(minimal_wind_unit + stage * wind_per_distance_to_neutral_point)
	var total_multiplier = added_multiplier_per_altitude * area.y
	var added_wind_for_altitude = total_multiplier * wind
	if wind > 0:
		added_wind_for_altitude = clamp(added_wind_for_altitude, -10, wind)
	else:
		added_wind_for_altitude = clamp(added_wind_for_altitude, wind, 10)

	wind -= added_wind_for_altitude
	var time_multiplier = get_time_multiplier()
	wind *= time_multiplier
	return wind

func get_time_multiplier() -> float:
	if WeatherGlobals.tick == null:
		return 1.0
	return sin(WeatherGlobals.tick.total_ticks * (PI / ticks_per_cycle))
