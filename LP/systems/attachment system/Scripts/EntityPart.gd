@tool
extends Bone2D
class_name EntityPart

#Entity Data
enum type {ANY, BODY, HEAD, APPENDAGE}
@export var thumbnail : Texture2D
@export var preview_name : String
@export var entity_type : type

var attached_socket : AttachmentSocket
var entity_snap_distance = 7 # Should be just shorter than snap distance in creaturecreator

#Movement tracking
var recently_moved = false
var recently_detached = false
var last_position : Vector2


#Activate notification system
func _ready() -> void:
	set_notify_transform(true)
	last_position = global_position


func snap_to_socket(socket : AttachmentSocket):
	if not socket or not socket.enabled:
		push_error("socket is null!")
		return
	
	global_position = socket.global_position
	recently_moved = false
	if socket.has_method("assign_new_limb"):
		socket.assign_new_limb(self)
		attached_socket = socket
	else:
		push_warning("socket lacks assign_limb function")


#This will trigger whenever the part is moved
func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSFORM_CHANGED:
		if global_position.distance_to(last_position) > entity_snap_distance:
			recently_moved = true
			last_position = global_position
			if attached_socket and attached_socket.my_entity == self:
				attached_socket.remove_limb()
