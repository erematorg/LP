extends Node
class_name Lightning
## Creates lightning effects, nothing currently depends on this.
## Dependent on [annotation Humidity.saturated_water_per_area] and [annotation AreaVisibility.shown_areas] nodes.

signal lightning_spawned

@export var lightning_steps:int
@export var branch_stroke_length:float
@export var initial_stroke_length:float

## Minimum amount of saturated water to spawn lightning
@export var minimum_lightning_saturation:float

## How much a lightning takes to fade away.
@export var lightning_duration:float

## The maximum distance between the lightning's origin and the surface being hit.
@export var maximum_ray_length:float

## from 0 to 100 how much chance should each tick have to spawn a lightning
@export var lightning_chance_per_tick:float

## This makes the main lightning line look less straight.
@export var initial_line_irregularities:float

## This makes the branches look less straight
@export var branch_irregularities:float

## Altitude in which to check if there's enough moisture for lightning to ocurr
@export var check_moisture_on:int

## From 0 to 100 how likely is a stroke to be divided into 2.
@export var branching_chance:float

## From 0 to 100 how likely is a branch to get another branch.
@export var secondary_branching_chance:float

## Debugging option that shows you when a branch starts and ends.
@export var show_direct_lines:bool

var last_direct_lines:Array[Array]


func _on_tick_timeout():
	if randf_range(0,100)<lightning_chance_per_tick:
		var can_ocurr=false
		for x in WeatherGlobals.area_visibility.visible_columns:
			var area_to_check_humidity=Vector2i(x,check_moisture_on)
			if (WeatherGlobals.rain_manager.is_raining_on_area(Vector2i(x,check_moisture_on+1)) and 
				WeatherGlobals.humidity.get_saturated_water(area_to_check_humidity)>=minimum_lightning_saturation):
					can_ocurr=true
		if can_ocurr:
			spawn_lightning()

## Creates a single lightning and it's branches.
func spawn_lightning():
	last_direct_lines.clear()
	var branches=generate_lightning()
	var width=12
	for branch in branches:
		var line=Line2D.new()
		line.points=branch
		add_child(line)
		line.width=width
		if width>4:
			width-=1
		get_tree().create_tween().tween_property(line,"modulate:a",0,lightning_duration).finished.connect(func():
			line.queue_free()
			)
	
	## For debugging purposes.
	if show_direct_lines:
		for direct in last_direct_lines:
			var line=Line2D.new()
			line.points=direct
			line.default_color=Color.VIOLET
			line.width=3
			add_child(line)
			get_tree().create_tween().tween_property(line,"modulate:a",0,lightning_duration).finished.connect(func():
				line.queue_free()
				)

## A line between the starting point and the ground, with some curves.
func get_lightning_initial_line(starting_point:Vector2)->Array[Vector2]:
	var points:Array[Vector2]=[starting_point]
	var falling_direction=Vector2.DOWN.rotated(randf_range(-PI/4,PI/4))
	var ending_point=starting_point+falling_direction*maximum_ray_length
	
	# We cast a ray towards ending point to check where the ray would hit.
	var space_state=get_parent().get_world_2d().direct_space_state
	var query=PhysicsRayQueryParameters2D.create(starting_point,ending_point)
	var result=space_state.intersect_ray(query)
	if not result.is_empty():
		# If the ray did hit something, changte the ending point.
		ending_point=result["position"]
	
	# We start drawing the way to the ending point in segments.
	var distance=initial_stroke_length
	var direction=starting_point.direction_to(ending_point)
	while distance<ending_point.distance_to(starting_point):
		var new_point=points[points.size()-1]+direction*initial_stroke_length
		points.append(new_point)
		distance+=initial_stroke_length
	
	points.append(ending_point)
	
	# Now we have a line from start to end in segments, lets add random curves to it along the way.
	var idx=1
	while idx<points.size()-1:
		points[idx]+=Vector2.LEFT.rotated(randf_range(-PI,PI))*randf_range(0,initial_line_irregularities)
		idx+=1
	
	return points

## Generates an array of lines (arrays of points) representing a lightning. if starting pos is Vector2.ZERO
## the position is automatically generated.
func generate_lightning(starting_pos:Vector2=Vector2.ZERO)->Array[Array]:
	# Define where the lightning starts
	if starting_pos==Vector2.ZERO:
		var camera_position=get_viewport().get_camera_2d().position
		var view_size=Vector2(get_viewport().size)/get_viewport().get_camera_2d().zoom
		starting_pos.x=randf_range(
				camera_position.x,
				camera_position.x+view_size.x
		)
		starting_pos.y=camera_position.y-100
	
	var initial_line=get_lightning_initial_line(starting_pos)
	
	var branches:Array[Array]=[initial_line]
	
	for point in initial_line:
		if randf_range(0,100)<branching_chance:
			var general_direction:Vector2
			if randf()<0.5:
				general_direction=Vector2.DOWN.rotated(randf_range(0.1,PI/4))
			else:
				general_direction=Vector2.DOWN.rotated(randf_range(-PI/4,-0.05))
			
			branches.append_array(get_branch(point,1,general_direction))
	
	return branches

## Gets a branch and it's own branches, as arrays of arrays containing points.
func get_branch(starting_point:Vector2,index:int,general_direction:Vector2)->Array[Array]:
	var current_branch:Array[Vector2]=[starting_point]
	var branches:Array[Array]=[current_branch]
	var current_stroke_size=branch_stroke_length/index

	
	var direction=general_direction.rotated(randf_range(-PI/3,PI/3))
	
	if direction.y<0:
		direction.y/=3
	direction=direction.normalized()
	
	var initial_length=lightning_steps*current_stroke_size
	var ending_point=starting_point+direction*initial_length
	
	# We cast a ray towards ending point to check where the ray would hit.
	var space_state=get_parent().get_world_2d().direct_space_state
	var query=PhysicsRayQueryParameters2D.create(starting_point,ending_point)
	var result=space_state.intersect_ray(query)
	if not result.is_empty():
		# If the ray did hit something, change the ending point.
		ending_point=result["position"]
	
	last_direct_lines.append([starting_point,ending_point])
	
	
	# We start drawing the way to the ending point in segments.
	for i in range(lightning_steps-2):
		var new_point=current_branch[current_branch.size()-1]+direction*current_stroke_size
		if new_point.distance_to(ending_point)>current_stroke_size:
			current_branch.append(new_point)
		else:
			break
	
	current_branch.append(ending_point)
	
	# Now we have a line from start to end in segments, lets add random curves to it along the way.
	var idx=1
	while idx<current_branch.size()-1:
		current_branch[idx]+=Vector2.LEFT.rotated(randf_range(-PI,PI))*randf_range(0,branch_irregularities/index+1)
		idx+=1
	
	if index<5:
		for point in current_branch:
			if randf_range(0,100)<secondary_branching_chance/index:
				branches.append_array(get_branch(point,index+1,direction))
		
	
	return branches


