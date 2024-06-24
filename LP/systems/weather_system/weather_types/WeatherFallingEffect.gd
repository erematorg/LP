extends Node2D
class_name WeatherFallingEffect
## Class used for wather effects that need to simulate falling particles which shouldn't dissapear

## Each emitter is assigned a width of the screen defined by this variable.
@export var water_column_size:float


## Emitters appear this much higher than the top of the screen
@export var emitter_top_margin:float

@export var particle_template:PackedScene


## Assigns a discrete cell to the camera using grid_size, used to know when to spawn more emitters.
var camera_grid_position : Vector2i
var particles_by_position: Dictionary
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
	var particles:ParticleProcessMaterial=emitter.process_material
	var speed_min=particles.initial_velocity_min
	var spawn_visibility_notifier:VisibleOnScreenNotifier2D = particle_scene.get_node("SpawnVisibilityNotifier")
	emitter.visibility_rect.position.y=0
	
	emitter.visibility_rect.position.x=-view_size.x
	emitter.visibility_rect.size.x=view_size.x*3
	spawn_visibility_notifier.screen_entered.connect(phase_out_emitter.bind(particle_scene,on_grid_position))
	emitter.visibility_rect.size.y=grid_size.y*3
	
	var hit_point = get_drop_colission_point(emitter.global_position,grid_size.y*3)
	
	if hit_point is Vector2:
		var drop_length:float = hit_point.y-particle_scene.position.y
		emitter.lifetime=drop_length/speed_min
		particle_scene.get_node("Splash").position.y=drop_length
		var line_points=particle_scene.get_node("Ray").points
		line_points.append(Vector2(grid_size.x/2,drop_length))
		particle_scene.get_node("Ray").points=line_points
	else:
		particle_scene.get_node("Splash").emitting=false
	
	particles_by_position[on_grid_position]=particle_scene


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

func get_rain_direction()->Vector3:
	var rotation_for_wind=-WeatherGlobals.wind.get_wind_on_area(WeatherUtilities.get_grid_position(get_viewport().get_camera_2d().get_screen_center_position()))/50
	var direction_with_wind=Vector2.DOWN.rotated(rotation_for_wind)
	return Vector3(direction_with_wind.x,direction_with_wind.y,0)

## Should return true if the effect should happen in this area. area is in the global grid.
func _is_area_needed(area:Vector2i)->bool:
	return true
