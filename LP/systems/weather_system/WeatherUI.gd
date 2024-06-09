extends Control

@onready var m_HumidityContainer: HBoxContainer = $VBoxContainer/HumidityContainer
@onready var m_MoistureContainer: HBoxContainer = $VBoxContainer/MoistureContainer
@onready var m_HeatContainer: HBoxContainer = $VBoxContainer/HeatContainer
@onready var m_WindContainer: HBoxContainer = $VBoxContainer/WindContainer
@onready var m_WeatherTypeDropdown: OptionButton = $VBoxContainer/WeatherTypeDropdown

var m_WeatherManager

func _ready():
	m_WeatherManager = get_node("/root/WeatherSystem/WeatherManager")

	if m_WeatherManager == null:
		print("WeatherManager node not found")
		return
	
	# Update slider values initially
	UpdateSliderValue(m_HumidityContainer, int(m_WeatherManager.humidity))
	UpdateSliderValue(m_MoistureContainer, int(m_WeatherManager.moisture))
	UpdateSliderValue(m_HeatContainer, int(m_WeatherManager.heat))
	UpdateSliderValue(m_WindContainer, int(m_WeatherManager.wind))

	# Populate weather type dropdown
	m_WeatherTypeDropdown.add_item("None", m_WeatherManager.WeatherState.NONE)
	m_WeatherTypeDropdown.add_item("Rain", m_WeatherManager.WeatherState.RAIN)
	m_WeatherTypeDropdown.add_item("Snow", m_WeatherManager.WeatherState.SNOW)
	m_WeatherTypeDropdown.add_item("Wind", m_WeatherManager.WeatherState.WIND)
	m_WeatherTypeDropdown.connect("item_selected", Callable(self, "_on_WeatherTypeDropdown_item_selected"))

func UpdateSliderValue(container: HBoxContainer, value: int):
	var slider: HSlider = container.get_node("HSlider")
	slider.set_value(value)

func UpdateLabelText(container: HBoxContainer, labelName: String, value: int):
	var label: Label = container.get_node("Label")
	label.set_text(labelName + " : " + str(value))

func _on_Humidity_value_changed(value):
	m_WeatherManager.update_weather_parameters(value, m_WeatherManager.moisture, m_WeatherManager.heat, m_WeatherManager.wind)
	UpdateLabelText(m_HumidityContainer, "Humidity", value)

func _on_Moisture_value_changed(value):
	m_WeatherManager.update_weather_parameters(m_WeatherManager.humidity, value, m_WeatherManager.heat, m_WeatherManager.wind)
	UpdateLabelText(m_MoistureContainer, "Moisture", value)

func _on_Heat_value_changed(value):
	m_WeatherManager.update_weather_parameters(m_WeatherManager.humidity, m_WeatherManager.moisture, value, m_WeatherManager.wind)
	UpdateLabelText(m_HeatContainer, "Heat", value)

func _on_Wind_value_changed(value):
	m_WeatherManager.update_weather_parameters(m_WeatherManager.humidity, m_WeatherManager.moisture, m_WeatherManager.heat, value)
	UpdateLabelText(m_WindContainer, "Wind", value)

func _on_WeatherTypeDropdown_item_selected(index: int):
	m_WeatherManager.change_weather(index)
