@tool
@icon("res://addons/attachmentgui/Sprites/socket.png")
extends Node2D
class_name AttachmentSocket

enum IK_chain_type {CCDIK, FABRIK}
@export var enabled : bool = false:
	set(val):
		enabled = val
		call_deferred("change_state")
@export var IK_type : IK_chain_type:
	set(val):
		IK_type = val
		request_ik_update.emit()
@export var accepted_type : EntityPart.type

#signals
signal request_ik_update
signal request_entity_reparent(socket)

#Occupancy
var my_entity : EntityPart

#Visuals
const GREEN_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/green_socket_small.png")
const RED_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/red_socket_small.png")
const GRAY_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/gray_socket_small.png")
const BLUE_SOCKET_SMALL = preload("res://addons/attachmentgui/Sprites/blue_socket_small.png")
@onready var sprite_2d: Sprite2D = $Sprite2D


func assign_new_limb(part : EntityPart):
	if not part or not enabled or part.entity_type != accepted_type:
		push_error("Cannot assign limb to socket")
		return
	my_entity = part
	update_state()
	if get_parent() and is_instance_valid(get_parent()):
		part.reparent(get_parent())


func remove_limb():
	if my_entity:
		my_entity.recently_detached = true
		my_entity = null
	update_state()


func change_state():
	if not enabled:
		if get_parent() != get_tree().edited_scene_root and is_instance_valid(get_tree().edited_scene_root):
			reparent(get_tree().edited_scene_root)
			my_entity = null
	else:
		request_entity_reparent.emit(self)
	update_state()


func update_state():
	if not sprite_2d:
		push_warning("Socket has no sprite!")
		return
	
	# Check if the socket is enabled
	if not enabled:
		sprite_2d.texture = GRAY_SOCKET_SMALL
		return

	# Check if socket is enabled, has an entity, and is parented correctly
	if my_entity and (get_parent() is Bone2D or get_parent() is EntityPart):
		sprite_2d.texture = BLUE_SOCKET_SMALL
	elif get_parent() is Bone2D:
		# If socket is enabled, attached to a bone, but has no entity
		sprite_2d.texture = GREEN_SOCKET_SMALL
	else:
		# Socket is enabled but not parented to a Bone2D or EntityPart
		sprite_2d.texture = RED_SOCKET_SMALL
