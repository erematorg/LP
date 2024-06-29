extends Timer
class_name Tick
## Orchestrates the speed in which the whole weather system operates.
## Also keeps count of the total ticks.

## Ticks since the game started.
var total_ticks:int=0

func _init():
	WeatherGlobals.tick=self

func _on_timeout():
	total_ticks+=1
