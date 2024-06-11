extends WeatherFallingEffect

# Snow-specific parameters
@export var snow_texture: Texture2D

func _ready():
	atlas_texture = snow_texture
	super._ready()

func _customize_emitter(emitter:GPUParticles2D):
	var process_material: ParticleProcessMaterial=emitter.process_material
	process_material.scale_min=4

func _on_weather_parameters_updated(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	# Adjust particle properties based on weather parameters
	pass
