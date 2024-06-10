extends WeatherModule

#Assigns a discrete cell using grid_size, used to know when to spawn more rain
var camera_grid_position : Vector2
var particles_by_position: Dictionary

# Rain-specific parameters
@export var rain_texture: Texture2D
# Grid used to assign different particle emitters to different areas.
# Should be in world coordinates.
@export var grid_size : Vector2

func _ready():
	position=Vector2.ZERO
	atlas_texture = rain_texture
	super._ready()

func _process(delta):
	camera_grid_position.x = round(camera.position.x/grid_size.x)
	camera_grid_position.y = floor(camera.position.y/grid_size.y)
	$CameraPosition.global_position = Vector2(camera_grid_position)*grid_size
	fill_rain_spaces()

func fill_rain_spaces():
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
			var new_particle=create_particle()
			new_particle.global_position=Vector2(i)*grid_size
			print(new_particle.global_position)
			particles_by_position[i]=new_particle
			print("created emitter")

func phase_out_emitter(emitter:GPUParticles2D):
	emitter.emitting=false
	await get_tree().create_timer( 15 * (grid_size.y/1080)).timeout
	emitter.queue_free()

func create_particle() -> Node2D:
	var particle = GPUParticles2D.new()
	var process_material = ParticleProcessMaterial.new()
	particle.process_material=process_material
	process_material.collision_mode=ParticleProcessMaterial.COLLISION_RIGID
	process_material.initial_velocity_min=400
	process_material.spread=2
	process_material.direction = Vector3(0, 1,0)
	process_material.gravity = Vector3(0, 20,0)
	process_material.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
	
	process_material.color=[Color.RED,Color.YELLOW,Color.GREEN,Color.CORNFLOWER_BLUE,Color.ALICE_BLUE,Color.MAGENTA,Color.BLACK,Color.CHOCOLATE].pick_random()
	particle.texture = atlas_texture
	# Displays more particles if we have to cover more screen, 1000 for a pixel perfect full hd viewport
	particle.speed_scale = 2.0
	particle.randomness = 0
	particle.explosiveness = 0
	particle.trail_enabled = true
	particle.trail_lifetime=0.04#s
	particle.trail_sections=2
	particle.trail_section_subdivisions=1
	particle.collision_base_size=2
	# For debugging purposes
	var debug_sprite=Sprite2D.new()
	debug_sprite.texture=load("res://logo.png")
	particle.add_child(debug_sprite)
	debug_sprite.scale=Vector2(4,4)
	
	#Set proportions
	particle.lifetime = grid_size.y/40
	particle.amount = (200)*(grid_size.x/1080)
	particle.visibility_rect.position=Vector2.ZERO
	process_material.emission_box_extents = Vector3(grid_size.x*0.5,1,0.0)
	process_material.emission_shape_offset.x=grid_size.x/2
	particle.visibility_rect.size=Vector2(grid_size.x*3,grid_size.y*10)
	
	add_child(particle)
	return particle
	
func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
