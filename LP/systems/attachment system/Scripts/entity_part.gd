@tool
extends Bone2D
class_name EntityPart

enum type {BODY, HEAD, APPENDAGE}
@export var thumbnail : Texture2D
@export var preview_name : String
@export var entity_type : type
var creator : CreatureCreator
var recently_moved = false
var closest_socket : AttachmentSocket


#Activate notification system
func _ready() -> void:
	set_notify_transform(true)


func inject_creature_creator(cc : CreatureCreator):
	creator = cc


func snap_to_socket(socket : AttachmentSocket):
	if not socket:
		push_error("socket is null!")
		return
	global_position = socket.global_position
	recently_moved = false
	if socket is AttachmentSocket:
		if socket.has_method("assign_new_limb"):
			socket.assign_new_limb(self)
		else:
			push_warning("socket lacks assign_limb function")
	else:
		print("Socket is NOT of type attachment_socket, it is: ", str(socket))


#This will trigger whenever the part is moved
func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSFORM_CHANGED:
		recently_moved = true
