extends Polygon2D
class_name Puddle

## Sadly we can't use preload for this, as it would cause a circular reference
var water_puddle:PackedScene=load("res://systems/weather_system/WatterPuddle.tscn")
var covered_areas :Array[Vector2i]
var colission:=ConcavePolygonShape2D.new()


func _ready():
	for i in polygon:
		var area=WeatherUtilities.get_grid_position(i+position)
		if not covered_areas.has(area):
			covered_areas.append(area)
	$Body/Shape.shape=colission
	update_colission()
	

## Amount should be in px squared.
func reduce(area:float):
	if is_queued_for_deletion():
		return
	
	ensure_flat_surface()
	
	var surface_points : Array=get_surface_points()
	var surface_point_indexes: Array[int]=[]
	for i in surface_points:
		var index=polygon.find(i)
		surface_point_indexes.append(index)
	
	# Determine height to remove
	var width=WeatherUtilities.num_distance(surface_points[0].x,surface_points[1].x)
	var height_to_remove=area/width
	var height=get_height()
	if height_to_remove<height:
		var x_ordered_polygon: Array = Array(polygon)
		x_ordered_polygon.sort_custom(is_rightmost_x)
		var polyline=Geometry2D.offset_polyline([Vector2(x_ordered_polygon[0].x-1,surface_points[0].y),Vector2(x_ordered_polygon[polygon.size()-1].x+1,surface_points[0].y)],height_to_remove*2)
		var new_polygons=Geometry2D.clip_polygons(
			polygon,
			polyline[0]
		)
		
		if new_polygons.is_empty():
			queue_free()
			return 
		
		polygon=new_polygons[0]
		var index=1
		while index<new_polygons.size():
			var new_body: Puddle = water_puddle.instantiate()
			var new_polygon=new_polygons[index]
			for i in range(new_polygon.size()):
				new_polygon[i] = new_polygon[i]+position
			new_body.polygon=PackedVector2Array(new_polygon)
			new_body.color=Color.BLUE
			get_parent().add_child(new_body)
			new_body.call_deferred("add_to_group","water_puddles")
			index+=1
			update_colission()
	else:
		queue_free()

func update_colission():
	var colission_polygon=polygon.duplicate()
	if colission_polygon.size() % 2 != 0:
		colission_polygon.append(polygon[polygon.size()-1])
	colission.segments=colission_polygon
	

##Ensure the top 2 points are in the same height
func ensure_flat_surface():
	# Ensure the top is flat, as real water
	var new_polygon=polygon
	var surface_points=get_surface_points()
	new_polygon[polygon.find(surface_points[0])].y=new_polygon[polygon.find(surface_points[1])].y
	polygon=new_polygon
	

func get_height():
	var points=Array(polygon)
	points.sort_custom(is_higher_y)
	return WeatherUtilities.num_distance(points[0].y,points[points.size()-1].y)

## Returns the 2 higher points of the puddle
func get_surface_points():
	var points=Array(polygon)
	points.sort_custom(is_higher_y)
	points.resize(2)
	return points

func is_higher_y(a,b):
	return a.y<b.y
func is_rightmost_x(a,b):
	return a.x>b.x

