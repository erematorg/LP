extends Node2D
class_name CloudDrawer
## Clouds are merely a visual representation of
## Humidity.saturated_water_per_area in the current area.

@export var circle_radius:float
@export var circle_sides:int
@export var cloud_primary_color:Color
@export var cloud_secondary_color:Color

var area:Vector2i = Vector2i.ZERO

@onready var humidity:Humidity = get_node("%Humidity")

func show_clouds()->void:
	show_cloud_screen(cloud_primary_color)
	show_cloud_screen(cloud_secondary_color)

func show_cloud_screen(color:Color):
	var cloud_circles=get_random_circles()
	var clouds:Array[PackedVector2Array]
	# Acummulated merging of circles
	var forming_cloud:PackedVector2Array=cloud_circles[0]
	for i in cloud_circles:
		var new_clouds=Geometry2D.merge_polygons(forming_cloud,i)[0]
		forming_cloud=new_clouds[0]
		new_clouds.remove_at(0)
		clouds.append_array(new_clouds)
	clouds.append(forming_cloud)
	for cloud in clouds:
		var new_cloud=Polygon2D.new()
		new_cloud.polygon=cloud
		new_cloud.color=color
		add_child(new_cloud)

## returns a list of circles in a different position
## Which can later be fused to draw clouds
func get_random_circles() ->Array[PackedVector2Array]:
	var circles : Array[PackedVector2Array]
	for i in round(humidity.get_saturated_water(area)/2):
		var new_circle=get_circle()
		var circle_position=Vector2(randf_range(0,WeatherGlobals.grid_size.x),randf_range(0,WeatherGlobals.grid_size.y))
		for index in range(new_circle.size):
			new_circle[index]+=circle_position
		circles.append(new_circle)
	return circles

func get_circle()->PackedVector2Array:
	var angle=0
	var angle_progression=(2*PI)/circle_sides
	var circle := PackedVector2Array([])
	for side in range(circle_sides):
		circle.append(Vector2.from_angle(angle)*circle_radius)
		angle+=angle_progression
	return circle
