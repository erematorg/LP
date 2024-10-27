@tool
@icon("res://systems/attachment system/socket.png")
extends Node2D
class_name AttachmentSocket

#Occupancy
@export var placement_mode : bool = true:
	set(val):
		placement_mode = val
		call_deferred("update_state")
var occupied : bool = false
var limb : LimbBase
var entity : EntityPart
#A possible way of constraining part types for each socket
#var accepted_type : int = EntityPart.type.BODY

#Visuals
var socket_icon = preload("res://systems/attachment system/socket.png")
const GREEN_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/green_socket_small.png")
const RED_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/red_socket_small.png")
const GRAY_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/gray_socket_small.png")
const BLUE_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/blue_socket_small.png")
@onready var sprite_2d: Sprite2D = $Sprite2D


func _ready() -> void:
	placement_mode = true #redundancy
	update_state()


func assign_new_limb(part : EntityPart):
	if not part or placement_mode:
		push_error("Cannot assign limb to socket")
		return
	occupied = true
	update_state()
	#NOTE, reparenting causes the node to exit tree for a frame and reenter
	part.reparent(get_parent())
	entity = part


func remove_limb():
	occupied = false
	entity = null
	update_state()


#func check_valid():
	#var parent = get_parent()
	#var v : bool = false
	#if parent and parent is Bone2D:
		#v = true
	#isValid = v


#Gray if placement, red if placed but empty, green if placed and socketed
func update_state():
	if placement_mode:
		sprite_2d.texture = GRAY_SOCKET_SMALL
		return
	if occupied:
		sprite_2d.texture = BLUE_SOCKET_SMALL
	else:
		sprite_2d.texture = GREEN_SOCKET_SMALL
		
