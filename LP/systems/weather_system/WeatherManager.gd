class_name WeatherManager
extends Node

# Enums for weather states
enum WeatherState {
	NONE,
	RAIN,
	SNOW,
	WIND
}

# Global weather parameters
var humidity: float = 50.0
var moisture: float = 50.0
var heat: float = 50.0
var wind: float = 0.0

# Function to update weather parameters (called by UI or other systems)
func update_weather_parameters(new_humidity: float, new_moisture: float, new_heat: float, new_wind: float):
	humidity = new_humidity
	moisture = new_moisture
	heat = new_heat
	wind = new_wind
	emit_signal("weather_parameters_updated", humidity, moisture, heat, wind)

# Signal to notify weather modules of parameter changes
signal weather_parameters_updated(humidity: float, moisture: float, heat: float, wind: float)

# Function to change the weather state
func change_weather(state: WeatherState):
	var weather_system = get_parent()
	if weather_system:
		weather_system.change_weather(state)