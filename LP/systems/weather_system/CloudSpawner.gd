extends Node
class_name CloudSpawner
## Crates cloud drawers
@export var cloud_drawer:PackedScene

var cloud_drawers: Dictionary = {}

func _on_area_visibility_area_shown(area):
	check_to_spawn(area)

func check_to_spawn(area:Vector2i):
	if WeatherGlobals.humidity.get_saturated_water(area)>4 and get_node("%AreaVisibility").shown_areas.has(area):
		if not cloud_drawers.has(area):
			spawn_cloud_on_area(area)

func spawn_cloud_on_area(area):
	var new_drawer=cloud_drawer.instantiate()
	new_drawer.area=area
	new_drawer.position=WeatherUtilities.get_real_position(area)
	add_child(new_drawer)
	new_drawer.call_deferred("show_clouds",0)
	cloud_drawers[area]=new_drawer

func _on_humidity_saturated_water(area):
	check_to_spawn(area)


func _on_area_visibility_area_hidden(area):
	if cloud_drawers.has(area) and is_instance_valid(cloud_drawers[area]):
		cloud_drawers[area].queue_free()
		cloud_drawers.erase(area)
