extends Node
class_name AreaVisibility
## Helper for other nodes, meant to notify them when a new area is made visible
## or hidden without populating the entire world with a visibility notifier.

signal area_shown(area:Vector2i)
signal area_hidden(area:Vector2i)

var shown_areas:Array[Vector2i]

func _init():
	WeatherGlobals.area_visibility=self

func _process(_delta):
	var new_shown_areas=get_visible_areas()
	for i in shown_areas:
		if not new_shown_areas.has(i):
			area_hidden.emit(i)
	shown_areas=new_shown_areas

func get_visible_areas()->Array[Vector2i]:
	# The real world view size divided by 2
	var offset=Vector2(get_viewport().size/2)/get_viewport().get_camera_2d().zoom
	
	var top_left_corner_position: Vector2 = get_viewport().get_camera_2d().get_screen_center_position()-offset
	var bottom_right_corner_position: Vector2 = get_viewport().get_camera_2d().get_screen_center_position()+offset
	var top_left_grid=WeatherUtilities.get_grid_position(top_left_corner_position)
	var bottom_right_grid=WeatherUtilities.get_grid_position(bottom_right_corner_position)
	
	var new_areas_shown:Array[Vector2i]=[]
	# 2 added on the end because range is exclusive on the end argument, and
	# because we need 1 grid square of margin on both sides.
	for x in range(top_left_grid.x-1,bottom_right_grid.x+2):
		for y in range(top_left_grid.y-1,bottom_right_grid.y+2):
			var position=Vector2i(x,y)
			new_areas_shown.append(position)
			if not shown_areas.has(position):
				area_shown.emit(position)
	

	return new_areas_shown
