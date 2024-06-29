extends WeatherModule

# Wind-specific parameters
@export var wind_texture: Texture2D

func _ready():
	atlas_texture = wind_texture
	super._ready()

func create_particle() -> Node2D:
	var particle = GPUParticles2D.new()
	particle.texture = atlas_texture
	particle.amount = 300
	particle.lifetime = 2.0
	particle.speed_scale = 1.0
	particle.direction = Vector2(1, 0)
	particle.gravity = Vector2(0, 0)
	particle.randomness = 0.6
	particle.explosiveness = 0.2
	add_child(particle)
	return particle