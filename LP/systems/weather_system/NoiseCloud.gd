extends Sprite2D

func _ready():
	update_size()
	get_viewport().size_changed.connect(update_size)

func update_size():
	texture.size=get_viewport_rect().size
