@tool
@icon("res://systems/attachment system/socket.png")
extends Node2D
class_name AttachmentSocket

#Occupancy
var placement_mode : bool = true
var isValid : bool = false
var occupied : bool = false
var limb : LimbBase
#A possible way of constraining part types for each socket
#var accepted_type : int = EntityPart.type.BODY

#Visuals
var socket_icon = preload("res://systems/attachment system/socket.png")
const GREEN_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/green_socket_small.png")
const RED_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/red_socket_small.png")
const GRAY_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/gray_socket_small.png")
@onready var sprite_2d: Sprite2D = $Sprite2D

func _ready() -> void:
	check_valid()
	update_state()
	
func assign_new_limb(part : EntityPart):
	check_valid()
	if not isValid:
		return
	if not part: #and !occupied:
		push_error("Part is missing!")
		return
	occupied = true
	update_state()
	#NOTE, reparenting causes the node to exit tree for a frame and reenter
	part.reparent(get_parent())


func remove_limb():
	occupied = false
	update_state()


func check_valid():
	var parent = get_parent()
	var v : bool = false
	if parent and parent is Bone2D:
		v = true
	isValid = v


func update_state():
	if not isValid:
		sprite_2d.texture = GRAY_SOCKET_SMALL
		return
	if occupied:
		sprite_2d.texture = RED_SOCKET_SMALL
	else:
		sprite_2d.texture = GREEN_SOCKET_SMALL
