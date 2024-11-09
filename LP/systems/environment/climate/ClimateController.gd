extends Node2D
class_name CLIMATE

# Declare MultiMeshes for different spheres
var troposphere_multimesh := MultiMesh.new()
var troposphere_instance : MultiMeshInstance2D

# Components (activated via inspector)
@export var thermodynamics_component_enabled: bool = true

# Grid configuration
@export var grid_size := 20
@export var cols := 40
@export var rows := 30

# Thermodynamics component and grids
var thermodynamics_component : ThermodynamicsComponent
var temperature_grid : Array = []
var heat_capacity_grid : Array = []

# Initialize scene
func _ready():
	# Initialize temperature and heat capacity grids
	_initialize_grids()
	_initialize_troposphere_multimesh()
	
	# Configure thermodynamics if enabled
	if thermodynamics_component_enabled:
		thermodynamics_component = ThermodynamicsComponent.new()
		add_child(thermodynamics_component)
		thermodynamics_component.set_grid(temperature_grid, heat_capacity_grid)

	# Initial update for the multimesh colors based on temperature
	_update_troposphere_multimesh_colors()

# Initialize temperature and heat capacity grids
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

# Initialize Troposphere MultiMesh for rendering the grid
func _initialize_troposphere_multimesh():
	troposphere_instance = MultiMeshInstance2D.new()
	troposphere_instance.name = "Troposphere"
	troposphere_instance.multimesh = troposphere_multimesh
	troposphere_multimesh.transform_format = MultiMesh.TRANSFORM_2D
	troposphere_multimesh.mesh = _create_particle_mesh()
	troposphere_multimesh.use_colors = true
	troposphere_multimesh.instance_count = cols * rows
	add_child(troposphere_instance)

# Create a simple particle mesh for visualization
func _create_particle_mesh() -> Mesh:
	var mesh = QuadMesh.new()
	mesh.size = Vector2(grid_size, grid_size)
	return mesh

# Handle input to add heat sources if enabled
func _input(event):
	if thermodynamics_component_enabled and event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var mouse_pos = event.position
		var col = int(mouse_pos.x / grid_size)
		var row = int(mouse_pos.y / grid_size)
		thermodynamics_component.add_heat_source(row, col, 50.0, 100.0)

# Update temperature grid and visuals
func _process(delta):
	if thermodynamics_component_enabled:
		thermodynamics_component.update_state()
	_update_troposphere_multimesh_colors()

# Update Troposphere MultiMesh colors based on temperature in thermodynamics_component
func _update_troposphere_multimesh_colors():
	for row in range(rows):
		for col in range(cols):
			var color: Color
			if thermodynamics_component_enabled:
				# Access the temperature value directly from the updated grid
				var temp_value = thermodynamics_component.temperature_grid[row][col]
				# Map the temperature to a color intensity for visualization
				var color_intensity = int(255 * (temp_value / thermodynamics_component.max_temperature))
				color = Color(color_intensity / 255.0, 0, 0)  # Red intensity based on temperature
			else:
				# Default color when no thermodynamics component is active
				color = Color(0.2, 0.2, 0.2)  # Neutral gray

			# Apply color and position to MultiMesh instance
			var instance_index = row * cols + col
			troposphere_multimesh.set_instance_transform_2d(
				instance_index, Transform2D(0, Vector2(col * grid_size, row * grid_size))
			)
			troposphere_multimesh.set_instance_color(instance_index, color)
