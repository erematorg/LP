extends Node2D

func _ready():
	update_pos()
	get_viewport().size_changed.connect(update_pos)

func update_pos():
	position=(get_viewport_rect().size/2)*get_parent().zoom
