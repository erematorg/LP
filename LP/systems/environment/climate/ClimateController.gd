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

# Thermodynamics component (only active if enabled)
var thermodynamics_component : ThermodynamicsComponent

# Initialize scene
func _ready():
	_initialize_troposphere_multimesh()

	# Initialize ThermodynamicsComponent if enabled
	if thermodynamics_component_enabled:
		thermodynamics_component = ThermodynamicsComponent.new()
		add_child(thermodynamics_component)
		thermodynamics_component.initialize_thermal_grid(cols, rows)

	_update_troposphere_multimesh_colors()

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

# Update temperature field
func _process(delta):
	if thermodynamics_component_enabled:
		thermodynamics_component.update_state()
	_update_troposphere_multimesh_colors()

# Update Troposphere MultiMesh colors based on temperature
func _update_troposphere_multimesh_colors():
	for row in range(rows):
		for col in range(cols):
			var color: Color
			if thermodynamics_component_enabled:
				var temp_value = thermodynamics_component.temperature_grid[row][col]
				var color_intensity = int(255 * (temp_value / thermodynamics_component.max_temperature))
				color = Color(color_intensity / 255.0, 0, 0)
			else:
				# Default color when no thermodynamics component is active
				color = Color(0.2, 0.2, 0.2)  # Neutral gray for visualizing the grid

			var instance_index = row * cols + col
			troposphere_multimesh.set_instance_transform_2d(instance_index, Transform2D(0, Vector2(col * grid_size, row * grid_size)))
			troposphere_multimesh.set_instance_color(instance_index, color)
