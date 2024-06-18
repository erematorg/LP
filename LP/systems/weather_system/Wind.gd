extends Node
## Distance between points with 0 wind.
@export var neutral_point_distance:int
## Minimal amount of wind when not 0, scaled when an area is further away of a neutral point
## or on higher altitudes. in px/tick
@export var minimal_wind_unit:float
## How much the amount of wind gets affected by the distance to a neutral point.
@export var wind_per_distance_to_neutral_point:float
## For each point of altitude, the wind gets added this percentage. 0.1 is 10%.
@export var added_multiplier_per_altitude:float
@export var x_change_per_tick:float
@export var max_wind:float

func get_wind_on_area(area:Vector2i):
	area=Vector2(area)
	area.x+=x_change_per_tick*WeatherGlobals.tick.total_ticks
	## The step of a transition between neutral points in which the area it's in.
	var neutral_point=floor(area.x/neutral_point_distance)*neutral_point_distance
	var stage=area.x-neutral_point
	var wind:=Vector2.ZERO
	if stage<neutral_point_distance/2:
		var distance_to_next_neutral_point=neutral_point_distance-stage
		wind.x=minimal_wind_unit+distance_to_next_neutral_point*wind_per_distance_to_neutral_point
	elif stage>=neutral_point_distance/2:
		wind.x=-(minimal_wind_unit+stage*wind_per_distance_to_neutral_point)
	wind.x=wind.x*added_multiplier_per_altitude*wind.y
	wind.x=clamp(wind.x,0,max_wind)
	return wind


