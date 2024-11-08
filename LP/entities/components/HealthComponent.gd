extends Node
class_name HealthComponent
##Temporary File only for testing purposes will later be way more enhanced for the best health system out there.
@export var max_health: float = 100.0
var current_health: float

func _ready():
	current_health = max_health

func take_damage(amount: float):
	current_health -= amount
	if current_health <= 0:
		die()

func die():
	print("Entity has died")
	queue_free()

func recover(amount: float):
	current_health = min(current_health + amount, max_health)
