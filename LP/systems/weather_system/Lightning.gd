extends Node

signal lightning_spawned

@export var lightning_steps:int
@export var lightning_stroke_length:float

## Minimum amount of saturated water to spawn lightning
@export var minimum_lightning_saturation:float

## from 0 to 100 how much chance should each tick have to spawn a lightning
@export var lightning_chance_per_tick:float

## Altitude in which to check if there's enough moisture for lightning to ocurr
@export var check_moisture_on:int

## From 0 to 100 how likely is a stroke to be divided into 2
@export var branching_chance:float



## Generates an array of lines (arrays of points) representing a lightning
func generate_lightning(starting_pos:Vector2=Vector2.ZERO)->Array[Array]:
	# Define where the lightning starts
	if starting_pos==Vector2.ZERO:
		var camera_position=get_viewport().get_camera_2d().position
		var view_size=Vector2(get_viewport().size)/get_viewport().get_camera_2d().zoom
		starting_pos.x=randf_range(
				camera_position.x+view_size.x/2,
				camera_position.x-view_size.x/2
		)
		starting_pos.y=(camera_position.y-view_size.y/2)-100
	
	var current_branch:Array[Vector2]=[starting_pos]
	var branches:Array[Array]=[current_branch]
	
	for i in range(lightning_steps):
		var next_point=current_branch[current_branch.size()-1]+Vector2.DOWN.rotated(randf_range(-PI/2,PI/2))*lightning_stroke_length
		current_branch.append(next_point)
		if randf_range(0,100)<branching_chance:
			branches.append_array(generate_lightning(next_point))
	return branches

func spawn_lightning():
	var branches=generate_lightning()
	for branch in branches:
		var line=Line2D.new()
		line.points=branch
		add_child(line)
		get_tree().create_tween().tween_property(line,"modulate:a",0,1).finished.connect(func():
			line.queue_free()
			)


func _on_tick_timeout():
	for x in WeatherGlobals.area_visibility.visible_columns:
		var area_to_check_humidity=Vector2i(x,check_moisture_on)
		if (WeatherGlobals.rain_manager.is_raining_on_area(Vector2i(x,check_moisture_on+1)) and 
				WeatherGlobals.humidity.get_saturated_water(area_to_check_humidity)>=minimum_lightning_saturation):
			if randf_range(0,100)<lightning_chance_per_tick:
				spawn_lightning()
