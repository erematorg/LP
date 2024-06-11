extends WeatherModule
class_name WeatherFallingEffect
## Class used for wather effects that need to simulate falling particles which shouldn't dissapear

## Grid used to assign different particle emitters to different areas.
## Should be in world coordinates.
@export var grid_size : Vector2 = Vector2(4000,4000)

## How much time should initial gravity be used
@export var initial_gravity_time:float=3
## Should be much higher than gravity, this is used to fill the screen with particles, only used by the time
## indicated by initial_gravity_time after creating an emitter
@export var initial_gravity : float = 400
@export var gravity : float = 30

## Assigns a discrete cell to the camera using grid_size, used to know when to spawn more emitters.
var camera_grid_position : Vector2
var particles_by_position: Dictionary

func _ready():
	position=Vector2.ZERO
	super._ready()

func _process(delta):
	camera_grid_position.x = floor(camera.position.x/grid_size.x)
	camera_grid_position.y = floor(camera.position.y/grid_size.y)
	fill_needed_spaces()

func fill_needed_spaces():
	var needed_positions=[
		Vector2(camera_grid_position+Vector2(1,-2)),
		Vector2(camera_grid_position+Vector2(-1,-2)),
		Vector2(camera_grid_position+Vector2(0,-2)),
		Vector2(camera_grid_position+Vector2(1,-1)),
		Vector2(camera_grid_position+Vector2(-1,-1)),
		Vector2(camera_grid_position+Vector2(0,-1)),
	]
	for filled_position in particles_by_position.keys():
		var particle_in_position: GPUParticles2D = particles_by_position[filled_position]
		if not needed_positions.has(filled_position):
			if filled_position.y >= camera_grid_position.y:
				phase_out_emitter(particle_in_position)
			else:
				particle_in_position.queue_free()
			particles_by_position.erase(filled_position)
		# Emitters shouldnt be on if we can see the rain generate
		if filled_position.y==camera_grid_position.y:
			particle_in_position.emitting=false
		else:
			particle_in_position.emitting=true
	
	for i in needed_positions:
		if not particles_by_position.has(i):
			var new_emitter = GPUParticles2D.new()
			var process_material = ParticleProcessMaterial.new()
			new_emitter.process_material=process_material
			process_material.spread=2
			process_material.direction = Vector3(0, 1,0)
			# At first the gravity is absurd to populate the screen with rain
			process_material.gravity = Vector3(0, initial_gravity,0)
			process_material.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
			process_material.collision_mode=ParticleProcessMaterial.COLLISION_RIGID
			
			#Set proportions
			new_emitter.lifetime = grid_size.y/50
			new_emitter.amount = (800)*(grid_size.x/1080)
			new_emitter.visibility_rect.position=Vector2.ZERO
			process_material.emission_box_extents = Vector3(grid_size.x*0.5,1,0.0)
			process_material.emission_shape_offset.x=grid_size.x/2
			new_emitter.visibility_rect.size=Vector2(grid_size.x*3,grid_size.y*10)
			
			#Taking into account 0 means at the left of the effect
			new_emitter.visibility_rect.position.x=-grid_size.x
			_customize_emitter(new_emitter)
			new_emitter.global_position=Vector2(i)*grid_size
			particles_by_position[i]=new_emitter
			get_tree().create_timer(initial_gravity_time).timeout.connect(adjust_gravity.bind(new_emitter))
			add_child(new_emitter)

func adjust_gravity(emitter):
	if not is_instance_valid(emitter):return
	emitter.process_material.gravity=Vector3(0,30,0)

func phase_out_emitter(emitter:GPUParticles2D):
	emitter.emitting=false
	await get_tree().create_timer( 15 * (grid_size.y/1080)).timeout
	emitter.queue_free()

## Should be overriden, changing the emitter's default values.
func _customize_emitter(emitter:GPUParticles2D) ->void:
	pass
