extends Timer
class_name Tick
## Orchestrates the speed in which the whole weather system operates.
## Also keeps count of the total ticks.

var total_ticks:int=0

func _init():
	WeatherGlobals.tick=self

func _on_timeout():
	total_ticks+=1
