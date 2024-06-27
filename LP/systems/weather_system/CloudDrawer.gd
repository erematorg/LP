extends Node2D
class_name CloudDrawer
## Clouds are merely a visual representation of
## Humidity.saturated_water_per_area in the current area.

@export var circle_radius:float
@export var circle_sides:int
@export var cloud_expansion_radius:float
@export var cloud_primary_color:Color
@export var cloud_secondary_color:Color
@export var max_clouds:int
## The size of cloud parts increases this much for every px squared of humidity
@export var size_change_per_humidity:float
## The amount of pixels squared of water that a cloud represents
@export var water_per_cloud:float
#How many circles make a cloud
@export var cloud_parts:int

var area:Vector2i = Vector2i.ZERO

@onready var humidity:Humidity = WeatherGlobals.humidity

func _ready():
	humidity.saturated_water.connect(show_clouds)

func show_clouds(_area)->void:
	var amount_of_water=round(humidity.get_saturated_water(area)-$Clouds.get_child_count()*water_per_cloud)
	var clouds_to_create=round(amount_of_water/water_per_cloud)
	if clouds_to_create>0 and $Clouds.get_child_count()<max_clouds:
		if humidity.get_saturated_water(area)>15:
			create_clouds(cloud_secondary_color,clouds_to_create/2)
			create_clouds(cloud_primary_color,clouds_to_create/2)
		else:
			create_clouds(cloud_primary_color,clouds_to_create)
			
	if (humidity.get_saturated_water(area)<40 and $Clouds.get_child_count()>max_clouds):
		await get_tree().create_tween().tween_property($Clouds.get_child(0),"modulate:a",0,2).finished
		$Clouds.get_child(0).queue_free()
	if clouds_to_create<0:
		for i in range(abs(clouds_to_create)):
			if $Clouds.get_child_count()>i:
				var cloud_to_delete=$Clouds.get_child(i)
				var tweener:PropertyTweener=get_tree().create_tween().tween_property(cloud_to_delete,"modulate:a",0,1)
				tweener.finished.connect(func():
					cloud_to_delete.queue_free()
				)

func create_clouds(color:Color,amount:int):
	var i=0
	while i<amount:
		var new_cloud=Polygon2D.new()
		var shape=get_cloud(cloud_parts)
		new_cloud.polygon=shape
		new_cloud.color=color
		new_cloud.modulate.a=0
		get_tree().create_tween().tween_property(new_cloud,"modulate:a",0.9,5)
		$Clouds.add_child(new_cloud)
		if randf_range(0,10)>5:
			var occluder=LightOccluder2D.new()
			var occluder_polygon=OccluderPolygon2D.new()
			occluder_polygon.polygon=shape
			occluder.occluder=occluder_polygon
			new_cloud.add_child(occluder)
		i+=1

func get_circle_radius()->float:
	return circle_radius+size_change_per_humidity*clamp(humidity.get_saturated_water(area),0,80)

## Returns a polygon representing a circle
func get_circle(circle_position:Vector2=Vector2.ZERO)->PackedVector2Array:
	var angle=0
	var angle_progression=(2*PI)/circle_sides
	var circle := PackedVector2Array([])
	for side in range(circle_sides):
		var radius= get_circle_radius()
		circle.append(Vector2.from_angle(angle)*radius+circle_position)
		angle+=angle_progression
	return circle

## Max size indicates the maximum amount of circles to use when building the cloud
func get_cloud(max_size:int)->PackedVector2Array:
	var cloud_position=Vector2(randf_range(0,WeatherGlobals.grid_size.x),randf_range(0,WeatherGlobals.grid_size.y))
	var parts=0
	var last_cloud:PackedVector2Array
	while parts<max_size:
		var fusion=Geometry2D.merge_polygons(last_cloud,get_circle(cloud_position))
		last_cloud=fusion[0]
		var offset=Vector2.from_angle(randf_range(0,2*PI))
		offset.y/=3
		cloud_position+=offset*get_circle_radius()/2
		parts+=1
	return last_cloud
