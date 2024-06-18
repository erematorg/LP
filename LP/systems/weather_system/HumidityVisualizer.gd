extends Node
## Draws humidity transmissions on the screen

@export var enabled:bool
@export var arrow:PackedScene

func _on_humidity_humidity_transmitted(from, to, amount):
	if not enabled:
		return
	
	var new_arrow:Polygon2D=arrow.instantiate()
	new_arrow.position=WeatherUtilities.get_real_position(from)
	if to.x==from.x:
		new_arrow.position.x+=WeatherGlobals.grid_size.x/2
	if to.y<from.y:
		new_arrow.rotation=-PI/2
	if to.y>from.y:
		new_arrow.position.y+=WeatherGlobals.grid_size.y
		new_arrow.rotation=PI/2
	add_child(new_arrow)
	new_arrow.get_node("Amount").text=str(amount)
	new_arrow.get_node("Arrow").color=Color.SKY_BLUE
	await get_tree().create_tween().tween_property(new_arrow,"modulate:a",0,1).finished
	new_arrow.queue_free()
