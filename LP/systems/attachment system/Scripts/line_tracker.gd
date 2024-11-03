@tool
extends Node
class_name LineTracker

@export var far_color = Color.RED
@export var close_color = Color.GREEN
var lines : Array[Line2D]
var snap_distance
var show_distance

func init_linetracker(snapdist, showdist):
	snap_distance = snapdist
	show_distance = showdist


func clear_old_lines():
	if lines.size() > 0:
		for line in lines:
			line.queue_free()
		lines.clear()


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
		var normalized_dist = clamp((dist - 0.0) / (show_distance - 0.0), 0.0, 1.0)
		line.width = lerp(2.5, 0.1, normalized_dist)
