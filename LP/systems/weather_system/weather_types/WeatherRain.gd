extends WeatherModule

# Rain-specific parameters
@export var rain_texture: Texture2D

func _ready():
	atlas_texture = rain_texture
	super._ready()

func create_particle() -> Node2D:
	var particle = GPUParticles2D.new()
	particle.texture = atlas_texture
	particle.amount = 1000
	particle.lifetime = 1.5
	particle.speed_scale = 2.0
	particle.direction = Vector2(0, 1)
	particle.gravity = Vector2(0, 800)
	particle.randomness = 0.5
	particle.explosiveness = 0.5
	add_child(particle)
	return particle

func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
