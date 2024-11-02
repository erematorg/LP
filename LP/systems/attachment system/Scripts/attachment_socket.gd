@tool
@icon("res://systems/attachment system/socket.png")
extends Node2D
class_name AttachmentSocket

enum IK_chain_type {CCDIK, FABRIK}
@export var placement_mode : bool = true:
	set(val):
		placement_mode = val
		call_deferred("update_state")
@export var IK_type : IK_chain_type:
	set(val):
		IK_type = val
		call_deferred("update_ik_type")
@export var accepted_type : EntityPart.type

var creature_creator : CreatureCreator

#Occupancy
#var limb : LimbBase ## deprecated
var my_entity : EntityPart

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


func init_cc(cc : CreatureCreator):
	if not creature_creator:
		creature_creator = cc


func assign_new_limb(part : EntityPart):
	if not part or placement_mode or part.entity_type != accepted_type:
		push_error("Cannot assign limb to socket")
		return
	my_entity = part
	if get_parent() and is_instance_valid(get_parent()):
		part.reparent(get_parent())
	update_state()


func remove_limb():
	my_entity.recently_detached = true
	my_entity = null
	update_state()


func find_closest_bone():
	if not creature_creator:
		push_warning("Creature Creator is null in socket!")
		return
	var closest_bone
	var closest_dist = INF
	for entity in creature_creator.entities:
		if my_entity == entity:
			continue
		var dist = global_position.distance_to(entity.global_position)
		if dist < closest_dist:
			closest_bone = entity
			closest_dist = dist
	if closest_bone:
		reparent(closest_bone)
		creature_creator.new_socket_in_scene(self)


func update_ik_type():
	if not creature_creator:
		return
	creature_creator.update_ik_types()
	

#Gray if placement, red if placed but empty, green if placed and socketed
func update_state():
	if placement_mode:
		sprite_2d.texture = GRAY_SOCKET_SMALL
		#Turn gray and unparent
		if not get_tree():
			return
		if is_instance_valid(get_tree().edited_scene_root):
			reparent(get_tree().edited_scene_root)
		return
	find_closest_bone()
	if my_entity:
		sprite_2d.texture = BLUE_SOCKET_SMALL
	else:
		if get_parent() is not Bone2D:
			sprite_2d.texture = RED_SOCKET_SMALL
		else:
			sprite_2d.texture = GREEN_SOCKET_SMALL
		
