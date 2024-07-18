extends Node2D

@onready var l_system_manager: LSystemManager = $LSystemManager
@onready var l_system_renderer: LSystemRenderer = $LSystemRenderer

func _ready():
	# Initialize the scene and set up the L-System manager and renderer
	l_system_manager.add_l_system()
	print("MainScene ready")
