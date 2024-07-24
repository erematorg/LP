extends Node2D

class_name WeatherSystem

enum WeatherState {
	NONE,
	RAIN,
	SNOW,
	WIND
}

@export var initial_weather_state: WeatherState = WeatherState.NONE
@export var weather_scenes: Array[PackedScene] = []

@export var weather_state_editor: WeatherState = WeatherState.NONE

var current_weather_state: WeatherState = WeatherState.NONE
var current_weather_system: Node2D = null
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

signal weather_changed(state: WeatherState)

func _ready():
	change_weather(initial_weather_state)
	set_weather_state(weather_state_editor)  # Ensure the editor value is used on startup

func change_weather(state: WeatherState):
	if current_weather_system:
		current_weather_system.queue_free()
		current_weather_system = null

	current_weather_state = state

	if state != WeatherState.NONE and state < weather_scenes.size():
		current_weather_system = weather_scenes[state].instantiate()
		add_child(current_weather_system)

	emit_signal("weather_changed", state)

func set_weather_state(state: WeatherState):
	if state != current_weather_state:
		change_weather(state)
	weather_state_editor = state  # Update the inspector value to reflect the change
