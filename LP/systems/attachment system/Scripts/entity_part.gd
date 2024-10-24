@tool
extends Bone2D
class_name EntityPart

enum type {BODY, HEAD, APPENDAGE}
@export var thumbnail : Texture2D
@export var preview_name : String
@export var entity_type : type
var creator : CreatureCreator
var recently_moved = false
var closest_socket : attachment_socket


#Activate notification system
func _ready() -> void:
	set_notify_transform(true)


func inject_creature_creator(cc : CreatureCreator):
	creator = cc


func snap_to_socket(socket : attachment_socket):
	global_position = socket.global_position
	recently_moved = false


#This will trigger whenever the part is moved
func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSFORM_CHANGED:
		recently_moved = true
