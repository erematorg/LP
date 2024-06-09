extends WeatherModule

# Snow-specific parameters
@export var snow_texture: Texture2D

func _ready():
	atlas_texture = snow_texture
	super._ready()

func create_particle() -> Node2D:
	var particle = GPUParticles2D.new()
	particle.texture = atlas_texture
	particle.amount = 500
	particle.lifetime = 3.0
	particle.speed_scale = 0.5
	particle.direction = Vector2(0, 1)
	particle.gravity = Vector2(0, 300)
	particle.randomness = 0.7
	particle.explosiveness = 0.3
	add_child(particle)
	return particle

func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
