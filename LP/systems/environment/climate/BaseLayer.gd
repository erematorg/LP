# Will be used later to separate hared Logic for Layers for common functionality for all atmospheric layers like Troposphe etc
extends Node
class_name BaseLayer

# Grid configuration
var grid_size := 20
var cols := 40
var rows := 30

# Grids for temperature and heat capacity
var temperature_grid : Array = []
var heat_capacity_grid : Array = []

# Initialize grids
func _initialize_grids():
	temperature_grid.clear()
	heat_capacity_grid.clear()
	for row in range(rows):
		var temp_row = []
		var capacity_row = []
		for col in range(cols):
			temp_row.append(0.0)
			capacity_row.append(1.0)  # Default heat capacity value
		temperature_grid.append(temp_row)
		heat_capacity_grid.append(capacity_row)
