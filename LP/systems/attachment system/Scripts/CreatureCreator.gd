@tool
extends Node2D
class_name CreatureCreator

#Socket snap settings
@export var snap_distance = 8.0
@export var show_line_distance = 25.0

# Dependencies
@export var creature_root: Skeleton2D
@export var m_tracker : MouseTracker
@export var line_tracker : LineTracker
@export var attachment_tracker : AttachmentTracker
@export var entity_tracker : EntityTracker
@export var component_node : Node


## This is our "start/ready" function, called from the attachmentgui
func inject_attachment_gui(gui : AttachmentGui):
	if not gui or not m_tracker or not line_tracker:
		push_error("Lacking required dependencies! - CretureCreator inject_attachment_gui")
		return
	connect_signals(gui)
	find_old_parts()
	attachment_tracker.ensure_socket_stack_pairs()
	attachment_tracker.update_stacks_with_occupied_parts()
	line_tracker.init_linetracker(snap_distance, show_line_distance)
	entity_tracker.init_tracker(creature_root, attachment_tracker, snap_distance, show_line_distance)


## Add and make sure signals are connected properly
func connect_signals(gui):
	if not gui.spawn_entity.is_connected(new_entity_in_scene):
		gui.spawn_entity.connect(new_entity_in_scene)
	if not gui.spawn_socket.is_connected(new_socket_in_scene):
		gui.spawn_socket.connect(new_socket_in_scene)
	if not gui.spawn_component.is_connected(new_component_in_scene):
		gui.spawn_component.connect(new_component_in_scene)
	if not gui.spawn_cosmetic.is_connected(new_cosmetic_in_scene):
		gui.spawn_cosmetic.connect(new_cosmetic_in_scene)
	if not m_tracker.stopped_dragging.is_connected(entity_tracker.drop_entity):
		m_tracker.stopped_dragging.connect(entity_tracker.drop_entity)


## New bodypart/entity in the scene
func new_entity_in_scene(entity : EntityPart):
	if not entity:
		return
	add_new_node(creature_root, entity)
	entity_tracker.new_entity(entity)


func new_socket_in_scene(socket : AttachmentSocket):
	if not socket:
		return
	add_new_node(self, socket)
	attachment_tracker.new_socket(socket)
	socket.request_entity_reparent.connect(find_close_entity)
	socket.request_ik_update.connect(attachment_tracker.update_ik_types)


func new_component_in_scene(component : String):
	# load component script
	var script = load(component) as Script
	if not script:
		push_warning("Component script failed to load - " + component)
		return
	#Check that the component already exists
	for c in component_node.get_children():
		if c.get_script() == script:
			push_warning("Component already exists!")
			return
	#create node
	var new_node = Node.new()
	new_node.set_script(script)
	add_new_node(component_node, new_node)
	new_node.name = script.get_global_name()+"_node"
	print("New node created and script attached from path:", component)


func new_cosmetic_in_scene(cosmetic : Sprite2D):
	add_new_node(self, cosmetic)


# Call a recursive function to search for parts if this is a previous scene
func find_old_parts():
	var root = get_tree().root
	search_for_parts(root)


#Recursive searching function
func search_for_parts(node: Node) -> void:
	if node is EntityPart:
		new_entity_in_scene(node)
	elif node is AttachmentSocket:
		new_socket_in_scene(node)
	# Recursively check all the children of this node
	for child in node.get_children():
		search_for_parts(child)


func will_process() -> bool:
	return Engine.is_editor_hint() and not (entity_tracker.entities.is_empty() or attachment_tracker.socket_stack_pairs.is_empty())

# Update positions of all entities and sockets, drawing lines between them
func _process(delta: float) -> void:
	if not will_process():
		return
	line_tracker.clear_old_lines()
	#Loop throuh each entity, if they have recently moved, prepare to draw lines to sockets
	for entity in entity_tracker.entities:
		if not entity.recently_moved:
			continue
		var closest_socket
		closest_socket = attachment_tracker.find_closest_socket(entity, show_line_distance)
		if closest_socket:
			if closest_socket.get_parent() == entity or entity.get_parent() == closest_socket:
				continue
			if entity.entity_type == closest_socket.accepted_type or closest_socket.accepted_type == EntityPart.type.ANY:
				line_tracker.draw_line_between(entity, closest_socket)


func add_new_node(parent, child):
	parent.add_child(child)
	child.owner = self


func find_close_entity(socket : AttachmentSocket):
	var closest_bone = null
	var closest_dist = INF
	var socket_pos = socket.global_position  # Cache the socket position

	for entity in entity_tracker.entities:
		# Skip if the entity is the same as the socket's entity
		if entity == socket.my_entity:
			continue
		
		# Calculate distance to current entity
		var dist = socket_pos.distance_to(entity.global_position)
		
		# Update closest entity if this one is closer
		if dist < closest_dist:
			closest_bone = entity
			closest_dist = dist

	# Attach to the closest entity if found
	if closest_bone:
		socket.reparent(closest_bone)
		attachment_tracker.new_socket(socket)
