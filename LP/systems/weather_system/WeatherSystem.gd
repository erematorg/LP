class_name WeatherSystem
extends Node2D

enum WeatherState {
	NONE,
	RAIN,
	SNOW,
	WIND
}

@export var initial_weather_state: WeatherState = WeatherState.NONE
@export var weather_scenes: Array[PackedScene]

var current_weather_state: WeatherState = WeatherState.NONE
var current_weather_system: Node2D = null
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

func _ready():
	change_weather(initial_weather_state)

func change_weather(state: WeatherState):
	if current_weather_system:
		current_weather_system.queue_free()
		current_weather_system = null

	current_weather_state = state

	if state != WeatherState.NONE and state < weather_scenes.size():
		current_weather_system = weather_scenes[state].instantiate()
		add_child(current_weather_system)
