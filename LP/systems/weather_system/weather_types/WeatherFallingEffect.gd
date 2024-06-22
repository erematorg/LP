extends Node2D
class_name WeatherFallingEffect
## Class used for wather effects that need to simulate falling particles which shouldn't dissapear

## Each emitter is assigned a width of the screen defined by this variable.
@export var water_column_size:float


## Emitters appear this much higher than the top of the screen
@export var emitter_top_margin:float

@export var particle_template:PackedScene

## How much time should initial gravity be used
@export var initial_gravity_time:float=3
## Should be much higher than gravity, this is used to fill the screen with particles, only used by the time
## indicated by initial_gravity_time after creating an emitter
@export var initial_gravity : float = 400
@export var gravity : float = 30
@export var max_height:int=-4
## Time between the emitter not being needed and it being deleted
@export var emitter_finishing_margin:float=10

@export var initial_speed:float
@export var normal_speed_min:float
@export var normal_speed_max:float

## Assigns a discrete cell to the camera using grid_size, used to know when to spawn more emitters.
var camera_grid_position : Vector2i
var particles_by_position: Dictionary
var grid_size:Vector2
var needed_positions : Array[Vector2i]

@onready var view_size=get_viewport_rect().size
func _ready():
	position=Vector2.ZERO
	get_viewport().size_changed.connect(update_grid_size)
	update_grid_size()

func update_grid_size():
	view_size=get_viewport_rect().size
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
	emitter.lifetime=grid_size.y*3/speed_min
	
	particles_by_position[on_grid_position]=particle_scene


func phase_out_emitter(container:Node2D,area:Vector2i):
	var deleted=particles_by_position.erase(area)
	var emitter:GPUParticles2D=container.get_node("Emitter")
	emitter.emitting=false
	get_tree().create_tween().tween_property(emitter,"modulate:a",0,emitter.lifetime)
	await get_tree().create_timer(emitter.lifetime).timeout
	if is_instance_valid(container):
		container.queue_free()


## Returns a list of vector2i's representing the areas where emitters need to be placed
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

## Should return true if the effect should happen in this area. area is in the global grid.
func _is_area_needed(area:Vector2i)->bool:
	return true

func adjust_gravity(emitter):
	if not is_instance_valid(emitter):return
	emitter.process_material.gravity=Vector3(0,30,0)


## Should be overriden, changing the emitter's default values.
func _customize_emitter(_emitter:GPUParticles2D,_for_position:Vector2i) ->void:
	pass
