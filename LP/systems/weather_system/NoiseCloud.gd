extends Sprite2D
class_name NoiseCloud

var shader:ShaderMaterial=material
var noise:FastNoiseLite=shader.get_shader_parameter("noise").noise
var humidities:={}
var x_offset:float

@onready var humidity := WeatherGlobals.humidity

func _ready():
	update_size()
	shader.set_shader_parameter("grid_size",WeatherGlobals.grid_size)
	get_viewport().size_changed.connect(update_size)

func _process(delta):
	shader.set_shader_parameter("position",global_position)
	if humidities.is_empty():
		x_offset=0
	else:
		var wind=WeatherGlobals.wind.get_wind_on_area(WeatherUtilities.get_grid_position(get_parent().get_screen_center_position()))
		x_offset-=wind*(delta/WeatherGlobals.tick.wait_time)
	shader.set_shader_parameter("texture_offset",x_offset)

func update_size():
	var view_size=get_viewport_rect().size/get_parent().zoom
	scale=view_size/64
	shader.set_shader_parameter("total_size",view_size)


func share_humidity_data():
	var humidity_values:=PackedFloat32Array()
	var humidity_positions:=PackedVector2Array()
	for i in humidities.keys():
		humidity_positions.append(i)
		humidity_values.append(humidities[i])
	shader.set_shader_parameter("humidities",humidity_values)
	shader.set_shader_parameter("humidity_grid_positions",humidity_positions)


func _on_area_visibility_area_shown(area):
	if humidity.get_saturated_water(area)>0:
		humidities[area]=humidity.get_saturated_water(area)
	share_humidity_data()


func _on_humidity_saturated_water(area):
	if WeatherGlobals.area_visibility.shown_areas.has(area):
		humidities[area]=humidity.get_saturated_water(area)
	share_humidity_data()


func _on_area_visibility_area_hidden(area):
	humidities.erase(area)
	share_humidity_data()
