extends Sprite2D

var shader:ShaderMaterial=material

func _ready():
	update_size()
	shader.set_shader_parameter("grid_size",WeatherGlobals.grid_size)
	get_viewport().size_changed.connect(update_size)

func _process(_delta):
	shader.set_shader_parameter("position",global_position)

func update_size():
	var view_size=get_viewport_rect().size/get_parent().zoom
	scale=view_size/64
	shader.set_shader_parameter("total_size",view_size)
