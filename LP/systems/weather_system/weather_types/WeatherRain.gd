extends WeatherFallingEffect
class_name WeatherRain

@export var drops_alpha_curve : CurveTexture
@export var debug_hints_enabled:bool = false

func _ready():
	super._ready()



func _customize_emitter(emitter:GPUParticles2D,_for_position:Vector2i) -> void:
	var process_material : ParticleProcessMaterial = emitter.process_material
	process_material.alpha_curve=drops_alpha_curve
	emitter.randomness = 0
	emitter.explosiveness = 0
	emitter.trail_enabled = true
	emitter.trail_lifetime=0.04#s
	emitter.trail_sections=2
	emitter.trail_section_subdivisions=1
	emitter.collision_base_size=2
	if debug_hints_enabled:
		process_material.color=[Color.RED,Color.YELLOW,Color.GREEN,Color.CORNFLOWER_BLUE,Color.ALICE_BLUE,Color.MAGENTA,Color.BLACK,Color.CHOCOLATE].pick_random()
		# For debugging purposes
		var debug_sprite=Sprite2D.new()
		debug_sprite.texture=load("res://logo.png")
		emitter.add_child(debug_sprite)
		debug_sprite.scale=Vector2(4,4)
