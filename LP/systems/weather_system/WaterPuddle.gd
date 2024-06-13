extends Polygon2D
class_name Puddle

var covered_areas :Array[Vector2i]

func _ready():
	for i in polygon:
		var area=WeatherUtilities.get_grid_position(i)
		if not covered_areas.has(area):
			covered_areas.append(area)

## Amount should be in px squared.
func reduce(area:float):
	if is_queued_for_deletion():
		return
	
	var surface_points : Array=get_surface_points()
	var surface_point_indexes: Array[int]=[]
	for i in surface_points:
		var index=polygon.find(i)
		surface_point_indexes.append(index)
	# Ensure the top is flat, as real water
	var new_polygon=polygon
	new_polygon[surface_point_indexes[0]].y=new_polygon[surface_point_indexes[1]].y
	polygon=new_polygon
	surface_points[0].y=surface_points[1].y

	var width=WeatherUtilities.num_distance(surface_points[0].x,surface_points[1].x)
	var height_to_remove=area/width
	
	if height_to_remove>get_height()+20:
		queue_free()
		return
	
	for i in surface_point_indexes:
		polygon[i].y+=height_to_remove
	
	var new_surface_point=get_surface_points()
	# We have to rely on index to check when the surface points get surpassed
	# because we already lowered said points.
	var new_surface_point_indexes=[polygon.find(new_surface_point[0]),polygon.find(new_surface_point[1])]
	if (not new_surface_point_indexes.has(surface_point_indexes[0])) or (not new_surface_point_indexes.has(surface_point_indexes[1])):
		divide(surface_point_indexes)

func get_height():
	var points=Array(polygon)
	points.sort_custom(is_higher_y)
	return WeatherUtilities.num_distance(points[0].y,points[1].y)

## Divides this puddle into smaller puddles that do not exceed the height of the current one.
func divide(last_surface_point_indexes:Array[int]):
	var last_surface_points: Array[Vector2]=[polygon[last_surface_point_indexes[0]],polygon[last_surface_point_indexes[1]]]
	#Indexes of points above last surface points
	var cutting_indexes:Array[int]=[]
	for index in range(polygon.size()):
		var point=polygon[index]
		if point.y<=last_surface_points[0].y and not last_surface_points.has(point):
			cutting_indexes.append(index)
	var index=last_surface_point_indexes.max()
	var new_polygons: Array[Array]=[[polygon[index]]]
	var polygon_index=0
	while true:
		index+=1
		if last_surface_point_indexes.has(index):
			new_polygons[polygon_index].append(polygon[index])
			break
		if index==polygon.size():
			index=0
		var current_polygon=new_polygons[polygon_index]
		if cutting_indexes.has(index):
			if current_polygon.size()<2:
				current_polygon.clear()
				continue
			
			var last_point:Vector2 = current_polygon[current_polygon.size()-1]
			##Ads the cut between the surface and the "island" point.
			current_polygon.append(Geometry2D.line_intersects_line(
					last_point,
					last_point.direction_to(polygon[index]),
					last_surface_points[0],
					last_surface_points[0].direction_to(last_surface_points[1])
			))
			polygon_index+=1
			new_polygons.append([])
			continue
		new_polygons[polygon_index].append(polygon[index])
	
	for new_polygon in new_polygons:
		var new_body=Puddle.new()
		for i in range(new_polygon.size()):
			new_polygon[i] = new_polygon[i]+position
		new_body.polygon=PackedVector2Array(new_polygon)
		new_body.color=Color.BLUE
		get_parent().add_child(new_body)
		new_body.call_deferred("add_to_group","water_puddles")
	queue_free()
## Returns the 2 higher points of the puddle
func get_surface_points():
	var points=Array(polygon)
	points.sort_custom(is_higher_y)
	points.resize(2)
	return points

func is_higher_y(a,b):
	return a.y<b.y

