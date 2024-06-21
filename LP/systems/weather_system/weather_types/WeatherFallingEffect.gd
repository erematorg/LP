extends WeatherModule
class_name WeatherFallingEffect
## Class used for wather effects that need to simulate falling particles which shouldn't dissapear

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

## Assigns a discrete cell to the camera using WeatherGlobals.grid_size, used to know when to spawn more emitters.
var camera_grid_position : Vector2
var particles_by_position: Dictionary

func _ready():
	WeatherGlobals.tick.timeout.connect(update_direction_with_wind)
	position=Vector2.ZERO
	super._ready()

func _process(_delta):
	camera_grid_position.x = floor(camera.position.x/WeatherGlobals.grid_size.x)
	camera_grid_position.y = floor(camera.position.y/WeatherGlobals.grid_size.y)
	fill_needed_spaces()

func fill_needed_spaces():
	var needed_positions=_get_needed_positions()
	for filled_position in particles_by_position.keys():
		var particle_in_position: GPUParticles2D = particles_by_position[filled_position]
		if needed_positions.has(filled_position):
			# Emitters shouldnt be on if we can see the rain generate
			if filled_position.y==camera_grid_position.y:
				particle_in_position.emitting=false
			else:
				particle_in_position.emitting=true
		else:
			phase_out_emitter(particle_in_position)
			particles_by_position.erase(filled_position)
	
	for i in needed_positions:
		if not particles_by_position.has(i):
			var new_emitter = GPUParticles2D.new()
			var process_material = ParticleProcessMaterial.new()
			new_emitter.process_material=process_material
			process_material.spread=2
			process_material.direction=Vector3.DOWN
			# At first the gravity is absurd to populate the screen with rain
			process_material.gravity = Vector3(0, initial_gravity,0)
			process_material.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
			if i.y>max_height:
				process_material.collision_mode=ParticleProcessMaterial.COLLISION_HIDE_ON_CONTACT
			process_material.direction=get_rain_direction(i)
			
			#Set proportions
			new_emitter.lifetime = WeatherGlobals.grid_size.y/50
			new_emitter.amount = (1000)*(WeatherGlobals.grid_size.x/1080)
			new_emitter.visibility_rect.position=Vector2.ZERO
			process_material.emission_box_extents = Vector3(WeatherGlobals.grid_size.x*0.5,1,0.0)
			process_material.emission_shape_offset.x=WeatherGlobals.grid_size.x/2
			new_emitter.visibility_rect.size=Vector2(WeatherGlobals.grid_size.x*3,WeatherGlobals.grid_size.y*10)
			
			process_material.initial_velocity_min=initial_speed
			get_tree().create_timer(2).timeout.connect(func():
				process_material.initial_velocity_min=normal_speed_min
				process_material.initial_velocity_max=normal_speed_max
			)
			#Taking into account 0 means at the left of the effect
			new_emitter.visibility_rect.position.x=-WeatherGlobals.grid_size.x
			_customize_emitter(new_emitter,i)
			new_emitter.global_position=Vector2(i)*WeatherGlobals.grid_size
			particles_by_position[i]=new_emitter
			get_tree().create_timer(initial_gravity_time).timeout.connect(adjust_gravity.bind(new_emitter))
			add_child(new_emitter)

func update_direction_with_wind():
	for area in particles_by_position.keys():
		particles_by_position[area].process_material.direction=get_rain_direction(area)
func get_rain_direction(area:Vector2i)->Vector3:
		var rotation_for_wind=-WeatherGlobals.wind.get_wind_on_area(area)/50
		var direction_with_wind=Vector2.DOWN.rotated(rotation_for_wind)
		return Vector3(direction_with_wind.x,direction_with_wind.y,0)
	

## Returns a list of vector2i's representing the areas where emitters need to be placed
func _get_needed_positions()->Array[Vector2i]:
	var needed_positions: Array[Vector2i] = WeatherGlobals.area_visibility.shown_areas
	for area in needed_positions.duplicate():
		if area.y<max_height:
			needed_positions.erase(area)
	return needed_positions
	
func adjust_gravity(emitter):
	if not is_instance_valid(emitter):return
	emitter.process_material.gravity=Vector3(0,30,0)

func phase_out_emitter(emitter:GPUParticles2D):
	emitter.emitting=false
	get_tree().create_tween().tween_property(emitter,"modulate:a",0,emitter_finishing_margin-1)
	get_tree().create_timer(emitter_finishing_margin).timeout.connect(func():
		emitter.queue_free()
		)

## Should be overriden, changing the emitter's default values.
func _customize_emitter(_emitter:GPUParticles2D,_for_position:Vector2i) ->void:
	pass
