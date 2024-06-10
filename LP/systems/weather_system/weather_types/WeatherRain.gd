extends WeatherModule

var particle: GPUParticles2D

# Rain-specific parameters
@export var rain_texture: Texture2D

func _ready():
	atlas_texture = rain_texture
	super._ready()

func create_particle() -> Node2D:
	particle = GPUParticles2D.new()
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
	update_emitter_proportions()
	get_viewport().size_changed.connect(update_emitter_proportions)
	return particle

func update_emitter_proportions():
	particle.process_material.emission_box_extents = Vector3(get_viewport_rect().size.x,1,0.0)
	particle.lifetime = 4 * (get_viewport_rect().size.x/1080)
	particle.amount = (200)*(get_viewport_rect().size.x/1080)

func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
