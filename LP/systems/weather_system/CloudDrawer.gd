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

var area:Vector2i = Vector2i.ZERO

@onready var humidity:Humidity = WeatherGlobals.humidity

func _ready():
	humidity.saturated_water.connect(show_clouds)

func show_clouds(_area)->void:
	var amount=round(humidity.get_saturated_water(area)-get_child_count()/2)
	if amount>0 and get_child_count()<max_clouds:
		show_cloud_screen(cloud_primary_color,amount)
		if humidity.get_saturated_water(area)>30:
			show_cloud_screen(cloud_secondary_color,amount/2)
	if humidity.get_saturated_water(area)<40 and get_child_count()>=max_clouds:
		await get_tree().create_tween().tween_property(get_child(0),"modulate:a",0,2).finished
		get_child(0).queue_free()

func show_cloud_screen(color:Color,amount:int):
	var i=0
	while i<amount/10:
		var new_cloud=Polygon2D.new()
		var shape=get_cloud(10)
		new_cloud.polygon=shape
		new_cloud.color=color
		new_cloud.modulate.a=0
		get_tree().create_tween().tween_property(new_cloud,"modulate:a",1,5)
		add_child(new_cloud)
		if randf_range(0,10)>5:
			var occluder=LightOccluder2D.new()
			var occluder_polygon=OccluderPolygon2D.new()
			occluder_polygon.polygon=shape
			occluder.occluder=occluder_polygon
			new_cloud.add_child(occluder)
		i+=1

## Returns a polygon representing a circle
func get_circle(circle_position:Vector2=Vector2.ZERO)->PackedVector2Array:
	var angle=0
	var angle_progression=(2*PI)/circle_sides
	var circle := PackedVector2Array([])
	for side in range(circle_sides):
		var radius=circle_radius+size_change_per_humidity*clamp(humidity.get_saturated_water(area),0,80)
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
		cloud_position+=offset*(circle_radius+size_change_per_humidity*clamp(humidity.get_saturated_water(area),0,80))/2
		parts+=1
	return last_cloud
