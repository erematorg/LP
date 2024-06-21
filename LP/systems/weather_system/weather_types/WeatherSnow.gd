extends WeatherFallingEffect


@onready var visibility:=WeatherGlobals.area_visibility

func _ready():
	super._ready()

func _customize_emitter(emitter:GPUParticles2D,_for_position):
	var process_material: ParticleProcessMaterial=emitter.process_material
	process_material.scale_min=4



func _get_needed_positions()->Array[Vector2i]:
	var adjacent_positions :Array[Vector2i]= super._get_needed_positions()
	var needed_positions: Array[Vector2i]=[]
	var needed_columns=visibility.visible_columns.duplicate()
	#  Add aditional columns for margin
	needed_columns.append_array([visibility.visible_columns.min()-1,visibility.visible_columns.max()+1])
	# Add necesary positions
	for x in visibility.visible_columns:
		var top=visibility.visible_rows.min()
		adjacent_positions.append(Vector2i(x,top-1))
	# Discard positions where it's not raining.
	for i in adjacent_positions:
		if WeatherGlobals.rain_manager.is_raining_on_area(i) and WeatherGlobals.temperature.get_temperature(i)<=0:
			needed_positions.append(i)
	# Discard positions in which we can see snow spawn
	for i in needed_positions.duplicate():
		if visibility.shown_areas_strict.has(i):
			needed_positions.erase(i)
	return needed_positions
