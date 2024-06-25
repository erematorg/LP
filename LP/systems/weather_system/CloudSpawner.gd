extends Node
class_name CloudSpawner
## Crates cloud drawers
@export var cloud_drawer:PackedScene
@export var noise:NoiseTexture2D

## Holds as keys areas, and as values lists of cloud drawers.
var cloud_drawers: Dictionary = {}

func _on_area_visibility_area_shown(area):
	check_to_spawn(area)

func check_to_spawn(area:Vector2i):
	if WeatherGlobals.humidity.get_saturated_water(area)>4 and WeatherGlobals.area_visibility.shown_areas.has(area):
		if not cloud_drawers.has(area):
			spawn_cloud_on_area(area)

func spawn_cloud_on_area(area):
	var new_drawer=cloud_drawer.instantiate()
	new_drawer.area=area
	new_drawer.position=WeatherUtilities.get_real_position(area)
	add_child(new_drawer)
	new_drawer.call_deferred("show_clouds",0)
	cloud_drawers[area]=[new_drawer]

func _on_humidity_saturated_water(area):
	check_to_spawn(area)


func _on_area_visibility_area_hidden(area):
	if cloud_drawers.has(area):
		for i in cloud_drawers[area]:
			remove_drawer(i,area)
			i.queue_free()

## deletes the drawer from the list, but doesn't free it.
func remove_drawer(drawer:CloudDrawer,area:Vector2i):
	if cloud_drawers.has(area):
		cloud_drawers[area].erase(drawer)
	else:
		printerr("Tried to remove a non-registered cloud drawer, something must have gone wrong.")
	if cloud_drawers[area].is_empty():
		cloud_drawers.erase(area)

func add_drawer(drawer:CloudDrawer,area:Vector2i):
	if cloud_drawers.has(area):
		cloud_drawers[area].append(drawer)
	else:
		cloud_drawers[area]=[drawer]
