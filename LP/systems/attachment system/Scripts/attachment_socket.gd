@tool
@icon("res://systems/attachment system/socket.png")
extends Node2D
class_name AttachmentSocket

var occupied : bool = false
#A possible way of constraining part types for each socket
#var accepted_type : int = EntityPart.type.BODY

var socket_icon = preload("res://systems/attachment system/socket.png")
const GREEN_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/green_socket_small.png")
const RED_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/red_socket_small.png")
var limb : LimbBase
@onready var sprite_2d: Sprite2D = $Sprite2D

func _ready() -> void:
	update_sprite()
	
func assign_new_limb(part):
	if part and !occupied:
		update_sprite()
		occupied = true
		#WARNING, reparenting causes the node to exit tree for a frame and reenter, causing issues
		#part.reparent(get_parent())

func remove_limb():
	pass

func update_sprite():
	if occupied:
		sprite_2d.texture = RED_SOCKET_SMALL
	else:
		sprite_2d.texture = GREEN_SOCKET_SMALL
