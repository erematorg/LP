@tool
extends Node2D
class_name CreatureCreator

#Socket snap line
@export var snap_distance = 8.0
@export var show_line_distance = 25.0
@export var far_color = Color.RED
@export var close_color = Color.GREEN

# Dependencies
@onready var creature_root: Skeleton2D = $CreatureRoot
@export var m_tracker : mouse_tracker

#Arrays
@export var entities : Array[EntityPart]
@export var sockets : Array[AttachmentSocket]
@export var lines : Array[Line2D]
var socket_pairs : Array[SocketModificationPair]
@export var socket_stack_pairs: Dictionary = {}

#We have to connect to the signal from this end
#So we inject the gui, and connect to its spawn signal
func inject_attachment_gui(gui : AttachmentGui):
	if not gui:
		push_error("gui is null in creature creator!")
		return
	if not gui.spawn_entity.is_connected(new_entity_in_scene):
		gui.spawn_entity.connect(new_entity_in_scene)
	if not gui.spawn_socket.is_connected(new_socket_in_scene):
		gui.spawn_socket.connect(new_socket_in_scene)
	

func new_entity_in_scene(entity : EntityPart):
	if not entity:
		push_error("Entity is null!")
		return
	add_entity_to_skeleton(entity)
	entities.push_back(entity)
	entity.tree_exited.connect(remove_entity.bind(entity))
	entity.tree_entered.connect(recall_entity.bind(entity))
	entity.inject_creature_creator(self)


func new_socket_in_scene(socket : AttachmentSocket):
	if not socket or sockets.has(socket):
		push_error("socket is invalid!")
		return

	sockets.push_back(socket)
	socket.tree_exited.connect(remove_socket.bind(socket))
	if not socket_stack_pairs.has(socket):
		add_stack_for_socket(socket)
	else:
		print("Socket already has a modification stack")
	print("new socket/stack pair added")
	socket.init_cc(self)
	ensure_socket_stack_pairs()


# Function to maintain pairs and keep them synchronized
func ensure_socket_stack_pairs():
	# Check for stale pairs (remove stacks if their socket no longer exists)
	for socket : AttachmentSocket in socket_stack_pairs.keys():
		if not sockets.has(socket):
			remove_stack_for_socket(socket)
			socket_stack_pairs.erase(socket)
	# Add stacks for new sockets
	for socket in sockets:
		if !socket_stack_pairs.has(socket):
			add_stack_for_socket(socket)
	# Warning if still not matched, but shouldn’t happen with this setup
	if creature_root.get_modification_stack().modification_count != socket_stack_pairs.size():
		print("Warning: Modifications and sockets are not fully synchronized.")


func add_stack_for_socket(socket: AttachmentSocket):
	# Only create a stack if the socket doesn't already have one
	var new_stack
	print("New socket with type: " + str(socket.IK_type))
	if socket.IK_type == AttachmentSocket.IK_chain_type.CCDIK:
		new_stack = SkeletonModification2DCCDIK.new()
	elif socket.IK_type == AttachmentSocket.IK_chain_type.FABRIK:
		new_stack = SkeletonModification2DFABRIK.new()
	creature_root.get_modification_stack().add_modification(new_stack)
	socket_stack_pairs[socket] = new_stack
	print("Added new stack for socket:", socket.name)


func update_ik_types():
	# Loop through each socket-modification pair in the dictionary
	for socket : AttachmentSocket in socket_stack_pairs.keys():
		var current_stack : SkeletonModification2D = socket_stack_pairs[socket]
		## Check if the stack type matches the socket's IK_type
		var correct_stack_type = null
		match socket.IK_type:
			socket.IK_chain_type.CCDIK:
				if not current_stack is SkeletonModification2DCCDIK:
					correct_stack_type = SkeletonModification2DCCDIK.new()
			socket.IK_chain_type.FABRIK:
				if not current_stack is SkeletonModification2DFABRIK:
					correct_stack_type = SkeletonModification2DFABRIK.new()

		## If we have determined stack is wrong. Change it
		if correct_stack_type:
			var modification_stack = creature_root.get_modification_stack()
			for i in modification_stack.modification_count:
				var i_stack = modification_stack.get_modification(i)
				if i_stack == current_stack:
					modification_stack.delete_modification(i)
					modification_stack.add_modification(correct_stack_type)
					socket_stack_pairs[socket] = correct_stack_type
					print("Stack updated")


func remove_stack_for_socket(socket: AttachmentSocket):
	# Remove the stack if it exists in the dictionary
	if socket_stack_pairs.has(socket):
		var stack_to_remove: SkeletonModification2D = socket_stack_pairs[socket]
		var modification_stack = creature_root.get_modification_stack()
		for i in creature_root.get_modification_stack().modification_count:
			var current_stack = modification_stack.get_modification(i)
			if current_stack == stack_to_remove:
				# Remove the modification at the found index
				modification_stack.delete_modification(i)
				print("Removed stack for socket:", socket.name)
				break  # Exit loop after deletion to avoid errors
		# Remove the socket entry from the dictionary
		socket_stack_pairs.erase(socket)
	else:
		push_error("Socket not found in socket_stack_pairs. No stack to remove.")


func add_entity_to_skeleton(entity : EntityPart):
	#Turn of the IK, and set pose, this is because the IK will go bananas if IK is active
	var stack : SkeletonModificationStack2D = creature_root.get_modification_stack()
	stack.enabled = false
	print("Please reset skeleton rest pose before enabling stack")


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	entities = []
	sockets = []
	lines = []
	if not m_tracker:
		push_error("mouse tracker is null!")
		return
	m_tracker.stopped_dragging.connect(drop_entity)
	find_old_parts()
	ensure_socket_stack_pairs()


func find_old_parts():
	var root = get_tree().root
	# Call a recursive function to search for the parts
	search_for_parts(root)
	print("Found entities: " + str(entities.size()))
	print("Found sockets: " + str(sockets.size()))


func search_for_parts(node: Node) -> void:
	# Add parts and sockets
	if node is EntityPart:
		new_entity_in_scene(node)
	elif node is AttachmentSocket:
		new_socket_in_scene(node)
	# Recursively check all the children of this node
	for child in node.get_children():
		search_for_parts(child)


func clear_old_lines():
	if lines.size() > 0:
		for line in lines:
			line.queue_free()
		lines.clear()


# Update positions of all entities and sockets, drawing lines between them
func _process(delta: float) -> void:
	#Runtime code here if needed
	
	#Only in editor code after this point:
	if not Engine.is_editor_hint():
		return

	clear_old_lines()
	if entities.is_empty() or sockets.is_empty():
		return
	#Loop throuh each entity, if they have recently moved, prepare to draw lines to sockets
	for entity in entities:
		if not entity.recently_moved:
			continue
		var closest_socket
		closest_socket = find_closest_socket(entity)
		if closest_socket:
			if entity.entity_type == closest_socket.accepted_type:
				draw_line_between(entity, closest_socket)


 #Find the closest socket to a given entity
func find_closest_socket(entity: EntityPart) -> AttachmentSocket:
	var closest_socket: AttachmentSocket = null
	var closest_dist = show_line_distance
	for socket in sockets:
		if socket.placement_mode or socket.occupied:
			continue
		var dist = entity.global_position.distance_to(socket.global_position)
		if dist < closest_dist:
			closest_dist = dist
			closest_socket = socket
	return closest_socket


# Draw a line between an entity and its closest socket, with dynamic appearance based on distance
func draw_line_between(entity: EntityPart, closest_socket: AttachmentSocket) -> void:
	var line = Line2D.new()
	set_line_visual(line, entity, closest_socket)
	line.add_point(entity.global_position)
	line.add_point(closest_socket.global_position)
	add_child(line)
	lines.push_back(line)


# Set line visual properties based on distance to the closest socket
func set_line_visual(line: Line2D, entity: EntityPart, closest_socket: AttachmentSocket) -> void:
	var dist = entity.global_position.distance_to(closest_socket.global_position)
	line.begin_cap_mode = Line2D.LINE_CAP_ROUND
	line.end_cap_mode = Line2D.LINE_CAP_ROUND
	if dist < snap_distance:
		line.default_color = close_color
		line.width = 3.0
	else:
		line.default_color = far_color
		var normalized_dist = clamp((dist - 0.0) / (show_line_distance - 0.0), 0.0, 1.0)
		line.width = lerp(2.5, 0.1, normalized_dist)


func remove_entity(entity : EntityPart):
	print("User removed an entity: " + entity.name)
	entities.erase(entity)
	
func recall_entity(entity : EntityPart):
	print("Part returned, reparented to: " + str(entity.get_parent()))
	entities.push_back(entity)

func remove_socket(socket : AttachmentSocket):
	print("User removed an entity: " + socket.name)
	remove_stack_for_socket(socket)
	sockets.erase(socket)
	socket.entity = null
	socket.update_state()
	ensure_socket_stack_pairs()


func drop_entity():
	for entity in entities:
		if not entity:
			push_warning("Entity is null! ")
			continue
		if entity.recently_moved:
			entity.recently_moved = false
			# Find closest socket if none is remembered
			var target_socket = entity.closest_socket
			if not target_socket:
				target_socket = find_closest_socket(entity)
			if not target_socket:
				print("no target socket")
				continue
			if entity.entity_type == target_socket.accepted_type: 
				try_snap(target_socket, entity)


func try_snap(target_socket : AttachmentSocket, entity):
	if not target_socket or not entity:
		return
	if target_socket.placement_mode:
		return
	# Snap to target socket if within range
	if target_socket and entity.global_position.distance_to(target_socket.global_position) < snap_distance:
		entity.snap_to_socket(target_socket)
		# Clear closest socket for future use
	entity.closest_socket = null
