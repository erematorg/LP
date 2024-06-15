extends Control

var selected_area:Vector2i=Vector2i.ZERO

@onready var humidity:Humidity = get_node("%Humidity")
@onready var temperature: Temperature = get_node("%Temperature")
@onready var area_indicator: Panel = get_node("%SelectedArea")

func _process(_delta):
	update_stats()
func _unhandled_input(event):
	if event is InputEventMouseButton:
		if event.button_index==MOUSE_BUTTON_LEFT:
			select_area(WeatherUtilities.get_grid_position(get_parent().get_parent().get_global_mouse_position()))

func update_stats():
	$Properties/Moisture/Value.value = humidity.get_air_humidity(selected_area)
	$Properties/MaxMoisture/Value.value = humidity.get_max_humidity(selected_area)
	$Properties/Temperature/Value.value = temperature.get_temperature(selected_area)
	$Properties/SaturatedMoisture/Value.value = humidity.get_saturated_water(selected_area)

func select_area(area:Vector2i):
	area_indicator.global_position=Vector2(area)*WeatherGlobals.grid_size
	area_indicator.size=WeatherGlobals.grid_size
	selected_area=area
	$Properties/Position.text=str(area)
