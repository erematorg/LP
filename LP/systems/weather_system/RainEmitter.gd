extends Node2D
class_name RainEmitter

@export var particle_process_material: ParticleProcessMaterial
@export var splash_material: ParticleProcessMaterial
var area: Vector2i

func _ready():
	var emitter = GPUParticles2D.new()
	emitter.process_material = particle_process_material
	emitter.amount = 20
	emitter.lifetime = 3.0
	emitter.trail_enabled = true
	emitter.trail_lifetime = 0.05
	add_child(emitter)

	var splash = GPUParticles2D.new()
	splash.process_material = splash_material
	splash.amount = 5
	splash.explosiveness = 0.1
	add_child(splash)
