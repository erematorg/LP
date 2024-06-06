extends Node2D

@export var population_dynamics: PopulationDynamics

func _process(delta):
	queue_redraw()

func _draw():
	for entity in population_dynamics.population:
		draw_circle(
			entity["position"] as Vector2, 
			(entity["genetics"] as GeneticAttributes).size, 
			(entity["genetics"] as GeneticAttributes).color
		)
