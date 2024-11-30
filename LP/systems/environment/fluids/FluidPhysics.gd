extends Node2D

var fluid_instance: MultiMeshInstance2D
var multimesh := MultiMesh.new()

@export var particle_count := 150
@export var boundary_size := 250
@export var particle_size := 6.0
@export var interaction_radius := 10.0
@export var rest_density := 1.5
@export var stiffness := 200.0  # Pressure stiffness
@export var viscosity := 0.1   # Viscosity factor
@export var velocity_damping := 0.97  # Velocity reduction upon collision
@export var mouse_interaction_radius := 100.0  # Radius for particle interaction with mouse

# Grid properties for optimization
var grid_size: float
var grid = {}
var grid_positions = []

# Particle properties
var velocities = []
var neighbors = []
var densities = []
var pressures = []
var prev_mouse_position = Vector2.ZERO

func _ready():
	_detect_multimesh_instance()
	calculate_grid_size()  # Calculate adaptive grid size
	initialize_multimesh()
	initialize_particles()
	apply_shader()  # Apply shader to particles

# Detect MultiMeshInstance2D node in the scene
func _detect_multimesh_instance():
	for child in get_children():
		if child is MultiMeshInstance2D and child.name == "FluidMultiMeshInstance2D":
			fluid_instance = child
			print("Detected FluidMultiMeshInstance2D.")

# Initialize the MultiMesh with particle mesh and set instance count
func initialize_multimesh():
	if fluid_instance:
		fluid_instance.multimesh = multimesh
		multimesh.transform_format = MultiMesh.TRANSFORM_2D
		multimesh.mesh = create_particle_mesh()
		fluid_instance.multimesh.instance_count = particle_count

# Create a mesh for each particle
func create_particle_mesh() -> Mesh:
	var particle_mesh := QuadMesh.new()
	particle_mesh.size = Vector2(particle_size, particle_size)
	return particle_mesh

# Apply a shader to the particles to visualize them better
func apply_shader():
	# Load the shader file
	var shader = load("res://shaders/Fluids.gdshader") as Shader
	var material = ShaderMaterial.new()
	material.shader = shader
	if fluid_instance:
		fluid_instance.material = material

# Calculate adaptive grid size based on interaction radius
func calculate_grid_size():
	grid_size = max(10.0, interaction_radius)

# Initialize particle positions, velocities, neighbors, densities, and pressures
func initialize_particles():
	velocities.resize(particle_count)
	neighbors.resize(particle_count)
	densities.resize(particle_count)
	pressures.resize(particle_count)
	grid_positions.resize(particle_count)

	for i in range(particle_count):
		var initial_pos = initialize_particle_position(i)
		set_particle_pos(i, initial_pos)
		velocities[i] = initialize_particle_velocity()

# Generate a random initial position for each particle
func initialize_particle_position(_index: int) -> Vector2:
	var random_x = randf_range(-boundary_size, boundary_size)
	var random_y = randf_range(-boundary_size, boundary_size)
	return Vector2(random_x, random_y)

# Generate a random initial velocity for each particle
func initialize_particle_velocity() -> Vector2:
	return Vector2(randf_range(-30, 30), randf_range(-30, 30))

# Set the position of a specific particle in the MultiMesh
func set_particle_pos(index: int, new_pos: Vector2):
	var local_transform := Transform2D()
	local_transform.origin = new_pos
	multimesh.set_instance_transform_2d(index, local_transform)

# Update grid with particle positions
func update_grid():
	grid.clear()
	for i in range(particle_count):
		var pos = multimesh.get_instance_transform_2d(i).origin
		var grid_pos = get_grid_cell(pos)
		grid_positions[i] = grid_pos
		if not grid.has(grid_pos):
			grid[grid_pos] = []
		grid[grid_pos].append(i)

# Get grid cell for a given position
func get_grid_cell(pos: Vector2) -> Vector2:
	return Vector2(floor(pos.x / grid_size), floor(pos.y / grid_size))

# Perform neighbor search using the grid
func update_neighbors():
	for i in range(particle_count):
		neighbors[i] = []
		var grid_pos = grid_positions[i]
		for x in range(-1, 2):
			for y in range(-1, 2):
				var neighbor_cell = grid_pos + Vector2(x, y)
				if grid.has(neighbor_cell):
					for j in grid[neighbor_cell]:
						if i != j:
							neighbors[i].append(j)

# Calculate densities and pressures for each particle
func calculate_density_and_pressure():
	for i in range(particle_count):
		var density = 0.0
		var pos_i = multimesh.get_instance_transform_2d(i).origin

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance < interaction_radius:
				density += (1 - distance / interaction_radius) ** 2  # Simplified kernel

		densities[i] = density
		pressures[i] = stiffness * max(0, densities[i] - rest_density)  # Simplified pressure

# Apply pressure forces to each particle
func apply_pressure_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		var pressure_force = Vector2.ZERO

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance < interaction_radius and distance > 0:
				var direction = (pos_i - pos_j).normalized()
				pressure_force += direction * (pressures[i] + pressures[j]) * (1 - distance / interaction_radius)

		velocities[i] += pressure_force * delta

# Apply viscosity forces to each particle
func apply_viscosity_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		var viscosity_force = Vector2.ZERO

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance < interaction_radius:
				var velocity_diff = velocities[j] - velocities[i]
				viscosity_force += velocity_diff * (1 - distance / interaction_radius)

		velocities[i] += viscosity * viscosity_force * delta

# Handle boundary collisions
func handle_boundary_collision(index: int, pos: Vector2):
	if pos.x < -boundary_size or pos.x > boundary_size:
		velocities[index].x = -velocities[index].x * velocity_damping
	if pos.y < -boundary_size or pos.y > boundary_size:
		velocities[index].y = -velocities[index].y * velocity_damping

	pos.x = clamp(pos.x, -boundary_size, boundary_size)
	pos.y = clamp(pos.y, -boundary_size, boundary_size)
	set_particle_pos(index, pos)

# Apply mouse interaction forces to particles
func apply_mouse_force(mouse_position: Vector2, prev_mouse_position: Vector2):
	var cursor_dx = mouse_position.x - prev_mouse_position.x
	var cursor_dy = mouse_position.y - prev_mouse_position.y

	for i in range(particle_count):
		var pos = multimesh.get_instance_transform_2d(i).origin
		var distance = pos.distance_to(mouse_position)

		if distance < mouse_interaction_radius:
			var strength = max(0, 1 - distance / mouse_interaction_radius)
			velocities[i].x += strength * cursor_dx
			velocities[i].y += strength * cursor_dy

func split_clipped_particles():
	for i in range(particle_count):
		for j in neighbors[i]:
			var pos_i = multimesh.get_instance_transform_2d(i).origin
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)

			if distance < particle_size * 1.5:  # Threshold for overlap
				resolve_clipping(i, j, distance)

func resolve_clipping(index_a: int, index_b: int, distance: float):
	if distance > 0:  # Avoid division by zero
		var pos_a = multimesh.get_instance_transform_2d(index_a).origin
		var pos_b = multimesh.get_instance_transform_2d(index_b).origin
		var direction = (pos_b - pos_a).normalized()
		var overlap = particle_size * 1.5 - distance
		var separation = direction * overlap * 0.5

		# Distribute separation
		pos_a -= separation
		pos_b += separation

		set_particle_pos(index_a, pos_a)
		set_particle_pos(index_b, pos_b)

func apply_repulsion_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance > 0 and distance < interaction_radius:
				var direction = (pos_i - pos_j).normalized()
				var overlap = interaction_radius - distance
				var repulsion_force = direction * overlap * 100.0  # Adjust strength as needed
				velocities[i] += repulsion_force * delta


# Main simulation loop
func _process(delta):
	update_grid()  # Update the grid for this frame
	update_neighbors()  # Find neighbors using the grid
	calculate_density_and_pressure()  # Compute densities and pressures
	apply_pressure_force(delta)  # Apply pressure forces
	apply_viscosity_force(delta)  # Apply viscosity forces
	apply_repulsion_force(delta)  # Apply repulsion force

	# Mouse interaction
	var mouse_pos = get_global_mouse_position()
	apply_mouse_force(mouse_pos, prev_mouse_position)
	prev_mouse_position = mouse_pos

	# Update particle positions and handle boundary collisions
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		pos_i += velocities[i] * delta
		handle_boundary_collision(i, pos_i)

	# Resolve any overlapping particles
	split_clipped_particles()
