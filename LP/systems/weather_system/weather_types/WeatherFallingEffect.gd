extends Node2D
class_name WeatherFallingEffect
## Class used for wather effects that need to simulate falling particles which shouldn't dissapear

## Each emitter is assigned a width of the screen defined by this variable.
@export var water_column_size:float


## Emitters appear this much higher than the top of the screen
@export var emitter_top_margin:float

@export var particle_template:PackedScene

@export var origin_height:int


## Assigns a discrete cell to the camera using grid_size, used to know when to spawn more emitters.
var camera_grid_position : Vector2i
var particles_by_position: Dictionary
var origin_emitters : Dictionary
var grid_size:Vector2
var needed_positions : Array[Vector2i]

@onready var view_size=get_viewport_rect().size/get_viewport().get_camera_2d().zoom
func _ready():
	position=Vector2.ZERO
	get_viewport().size_changed.connect(update_grid_size)
	update_grid_size()
	WeatherGlobals.tick.timeout.connect(tick)

func tick():
	for area in particles_by_position.keys():
		particles_by_position[area].get_node("Emitter").process_material.direction=get_rain_direction()

func update_grid_size():
	for i in get_children():
		i.queue_free()
	particles_by_position.clear()
	origin_emitters.clear()
	view_size=get_viewport_rect().size/get_viewport().get_camera_2d().zoom
	var new_grid_size=Vector2(water_column_size,view_size.y)
	if new_grid_size!=grid_size:
		grid_size=new_grid_size

func _process(delta):
	camera_grid_position=(get_viewport().get_camera_2d().position/grid_size).floor()
	needed_positions=_get_needed_positions()
	for area in needed_positions:
		if not particles_by_position.has(area):
			add_emitter(area)
	for area in particles_by_position.keys():
		if not needed_positions.has(area):
			if is_instance_valid(particles_by_position[area]):
				phase_out_emitter(particles_by_position[area],area)
	
	# Now we check if we have to draw where the particles begin to fall
	var needed_origin_emitters:Array[Vector2i] = []
	var global_camera_grid_position = WeatherUtilities.get_grid_position(get_viewport().get_camera_2d().global_position)
	var start_y_global=global_camera_grid_position.y
	var end_y_global=start_y_global+ceil(view_size.y/WeatherGlobals.grid_size.y)
	var local_origin_height=floor((origin_height*WeatherGlobals.grid_size.y)/grid_size.y)
	if origin_height>=start_y_global and origin_height<=end_y_global:
		var start_x=get_camera_grid_position().x-(view_size/grid_size).floor().x
		var end_x=get_camera_grid_position().x+(view_size/grid_size).floor().x*2
		for x in range(start_x-1,end_x+2):
			if _is_area_needed(Vector2i(floor(x*grid_size.x/WeatherGlobals.grid_size.x),origin_height)):
				needed_origin_emitters.append(Vector2i(x*grid_size.x,origin_height*WeatherGlobals.grid_size.y))
	for i in origin_emitters.keys():
		if not needed_origin_emitters.has(i):
			origin_emitters.erase(i)
	for i in needed_origin_emitters:
		if not origin_emitters.has(i):
			var emitter=create_origin_emitter(i)
			origin_emitters[i]=emitter

func create_origin_emitter(on_position:Vector2)->Node2D:
	var particle_scene:Node2D = particle_template.instantiate()
	particle_scene.position=on_position
	
	add_child(particle_scene)
	
	var emitter:GPUParticles2D=particle_scene.get_node("Emitter")
	emitter.process_material.emission_box_extents.y = WeatherGlobals.grid_size.y
	emitter.process_material.emission_shape_offset.y = WeatherGlobals.grid_size.y
	adjust_visibility_rect(emitter)
	adjust_fall_length(particle_scene)
	
	return particle_scene

func get_camera_grid_position()->Vector2i:
	return Vector2i((get_viewport().get_camera_2d().position/grid_size).floor())


## on_grid_position is in the rain grid.
func add_emitter(on_grid_position:Vector2i):
	var x = on_grid_position.x*grid_size.x + water_column_size/2
	var particle_scene:Node2D = particle_template.instantiate()
	particle_scene.position.x=x
	particle_scene.position.y=on_grid_position.y*grid_size.y - emitter_top_margin
	
	
	add_child(particle_scene)
	var emitter:GPUParticles2D = particle_scene.get_node("Emitter")
	var spawn_visibility_notifier:VisibleOnScreenNotifier2D = particle_scene.get_node("SpawnVisibilityNotifier")
	adjust_visibility_rect(emitter)
	spawn_visibility_notifier.screen_entered.connect(phase_out_emitter.bind(particle_scene,on_grid_position))
	
	adjust_fall_length(particle_scene)
	
	particle_scene.area=on_grid_position
	particles_by_position[on_grid_position]=particle_scene

func adjust_fall_length(emitter_container:Node2D):
	var emitter:GPUParticles2D=emitter_container.get_node("Emitter")
	var hit_point = get_drop_colission_point(emitter.global_position,grid_size.y*3)
	
	var speed_min:float=emitter.process_material.initial_velocity_min
	if hit_point is Vector2:
		var drop_length:float = hit_point.y-emitter_container.position.y
		emitter.lifetime=drop_length/speed_min
		emitter_container.get_node("Splash").position.y=drop_length
		var line_points=emitter_container.get_node("Ray").points
		line_points.append(Vector2(grid_size.x/2,drop_length))
		emitter_container.get_node("Ray").points=line_points
	else:
		emitter_container.get_node("Splash").emitting=false
		emitter.lifetime=grid_size.y/speed_min
	

func adjust_visibility_rect(emitter:GPUParticles2D)->void:
	emitter.visibility_rect.position.y=0
	emitter.visibility_rect.position.x=-view_size.x
	emitter.visibility_rect.size.x=view_size.x*3
	emitter.visibility_rect.size.y=grid_size.y*3
	

func get_drop_colission_point(on_position:Vector2,reach:float):
	var space = get_world_2d().space
	var space_state=PhysicsServer2D.space_get_direct_state(space)
	var query=PhysicsRayQueryParameters2D.create(on_position,on_position+Vector2.DOWN*reach)
	var result=space_state.intersect_ray(query)
	if result.is_empty():
		return false
	else:
		return result["position"]

func phase_out_emitter(container:Node2D,area:Vector2i):
	var deleted=particles_by_position.erase(area)
	var emitter:GPUParticles2D=container.get_node("Emitter")
	emitter.emitting=false
	get_tree().create_tween().tween_property(emitter,"modulate:a",0,emitter.lifetime)
	await get_tree().create_timer(emitter.lifetime).timeout
	if is_instance_valid(container):
		container.queue_free()


## Returns a list of vector2i's representing the areas where emitters need to be placed. Areas in local grid.
func _get_needed_positions()->Array[Vector2i]:
	var needed_positions: Array[Vector2i] = []
	var start_x=get_camera_grid_position().x-(view_size/grid_size).floor().x
	var end_x=get_camera_grid_position().x+(view_size/grid_size).floor().x*2
	for x in range(start_x-1,end_x+2):
		
		var current_position=Vector2i(x,get_camera_grid_position().y)
		var global_grid_position=WeatherUtilities.get_grid_position(Vector2(current_position)*grid_size)
		if _is_area_needed(global_grid_position):
			needed_positions.append(current_position)
			needed_positions.append(current_position+Vector2i.UP)
		
	
	return needed_positions

func global_grid_to_local_grid(global_grid_position):
	if global_grid_position is float:
		return floor((global_grid_position*WeatherGlobals.grid_size.y)/grid_size.y)

func get_rain_direction()->Vector3:
	var rotation_for_wind=-WeatherGlobals.wind.get_wind_on_area(WeatherUtilities.get_grid_position(get_viewport().get_camera_2d().get_screen_center_position()))/50
	var direction_with_wind=Vector2.DOWN.rotated(rotation_for_wind)
	return Vector3(direction_with_wind.x,direction_with_wind.y,0)

## Should return true if the effect should happen in this area. area is in the global grid.
func _is_area_needed(_area:Vector2i)->bool:
	return true
