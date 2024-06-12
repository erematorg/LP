extends Polygon2D

var covered_areas :Array[Vector2i]

func _ready():
	for i in polygon:
		var area=WeatherUtilities.get_grid_position(i)
		if not covered_areas.has(area):
			covered_areas.append(area)

## Amount should be in px squared to reduce.
func reduce(area:float):
	var surface_points : Array=get_surface_points()
	var width
	if sign(surface_points[0].x)!=sign(surface_points[1].x):
		width=abs(surface_points[0].x)+abs(surface_points[1].x)
	else:
		width=abs(surface_points[0].x-surface_points[1].x)
	var height_to_remove=area/width
	for i in surface_points:
		var index=polygon.find(i)
		polygon[index].y+=height_to_remove

## Returns the 2 higher points of the puddle
func get_surface_points():
	var points=Array(polygon)
	points.sort_custom(is_higher_y)
	points.resize(2)
	return points

func is_higher_y(a,b):
	return a.y<b.y

