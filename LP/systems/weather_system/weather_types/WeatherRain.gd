extends WeatherModule

#Assigns a discrete cell using grid_size, used to know when to spawn more rain
var camera_grid_position : Vector2i
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
	camera_grid_position.y = round(camera.position.y/grid_size.y)
	$CameraPosition.global_position = Vector2(camera_grid_position)*grid_size
	fill_rain_spaces()

func fill_rain_spaces():
	var needed_positions=[
		Vector2i(camera_grid_position+Vector2i(1,0)),
		Vector2i(camera_grid_position+Vector2i(-1,0)),
		camera_grid_position
	]
	for filled_position in particles_by_position.keys():
		if not needed_positions.has(filled_position):
			particles_by_position[filled_position].queue_free()
			particles_by_position.erase(filled_position)
	for i in needed_positions:
		if not particles_by_position.has(i):
			var new_particle=create_particle()
			new_particle.position=Vector2(i)*grid_size
			particles_by_position[i]=new_particle

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
	particle.texture = atlas_texture
	# Displays more particles if we have to cover more screen, 1000 for a pixel perfect full hd viewport
	particle.speed_scale = 2.0
	particle.randomness = 0.5
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
	
	update_emitter_proportions(particle)
	add_child(particle)
	return particle

func update_emitter_proportions(particle):
	particle.position.x=grid_size.x/2
	particle.process_material.emission_box_extents = Vector3(grid_size.x,1,0.0)
	particle.lifetime = 4 * (grid_size.y/1080)
	particle.amount = (200)*(grid_size.x/1080)
	particle.visibility_rect.size=grid_size
	particle.visibility_rect.position.x=-grid_size.x/2

func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
