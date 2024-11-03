@tool
extends Node
class_name EntityTracker

var attachment_tracker : AttachmentTracker
var skeleton
var entities : Array[EntityPart]
var snap_dist
var max_line_dist

func init_tracker(skel, attachtracker, snapdist, linedist):
	skeleton = skel
	attachment_tracker = attachtracker
	snap_dist = snapdist
	max_line_dist = linedist

func new_entity(entity : EntityPart):
	if not entity:
		push_error("Entity is null!")
		return
	entities.push_back(entity)
	if not entity.tree_exited.is_connected(remove_entity):
		entity.tree_exited.connect(remove_entity.bind(entity))
	if not entity.tree_entered.is_connected(recall_entity):
		entity.tree_entered.connect(recall_entity.bind(entity))

	
func remove_entity(entity : EntityPart):
	print("User removed an entity: " + entity.name)
	entities.erase(entity)
	
func recall_entity(entity : EntityPart):
	print("Part returned, reparented to: " + str(entity.get_parent()))
	entities.push_back(entity)
	
func drop_entity():
	for entity in entities:
		if not entity or not entity.recently_moved:
			continue
		if entity.recently_detached:
			entity.reparent(skeleton)
			entity.attached_socket.remove_limb()
			entity.recently_detached = false
		entity.recently_moved = false
		# Find closest socket if none is remembered
		var target_socket = entity.closest_socket if entity.closest_socket else attachment_tracker.find_closest_socket(entity, max_line_dist)
		if not target_socket or target_socket.enabled:
			continue
		if target_socket.get_parent() == entity or entity.get_parent() == target_socket:
			continue
		if entity.entity_type == target_socket.accepted_type: 
			try_snap(target_socket, entity)
	attachment_tracker.update_stacks_with_occupied_parts()
	
func try_snap(target_socket : AttachmentSocket, entity : EntityPart):
	# Snap to target socket if within range
	if target_socket and entity.global_position.distance_to(target_socket.global_position) < snap_dist:
		entity.snap_to_socket(target_socket)
	# Clear closest socket for future use
	entity.closest_socket = null
