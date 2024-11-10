extends Node2D

var fluid_instance: MultiMeshInstance2D
var multimesh := MultiMesh.new()

@export var particle_count := 200
@export var boundary_size := 250
@export var particle_size := 6.0
@export var interaction_radius := 10.0
@export var rest_density := 3
@export var stiffness := 500.0 # Pressure stiffness
@export var viscosity := 0.1 # Viscosity factor
@export var velocity_damping := 1.0 # Velocity reduction upon collision
@export var cohesion := 0.15 # Cohesion factor
@export var surface_tension := 0.1 # Surface tension factor
@export var restoring_factor := 0.02 # Restoring force strength
@export var mouse_interaction_radius := 75 # Radius for particle interaction with mouse
@export var grid_size := 20.0  # Size of each grid cell

# Particle properties
var velocities = []
var neighbors = []
var densities = []
var pressures = []
var initial_positions = []  # Store the initial positions of particles
var prev_mouse_position = Vector2.ZERO
var grid = {}  # Grid structure for neighbor optimization

func _ready():
	_detect_multimesh_instance()
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
	var shader = load("res://shaders/Fluids.gdshader") as Shader
	var material = ShaderMaterial.new()
	material.shader = shader
	if fluid_instance:
		fluid_instance.material = material

# Initialize particle positions, velocities, neighbors, densities, and pressures
func initialize_particles():
	velocities.resize(particle_count)
	neighbors.resize(particle_count)
	densities.resize(particle_count)
	pressures.resize(particle_count)
	initial_positions.resize(particle_count)

	for i in range(particle_count):
		var initial_pos = initialize_particle_position(i)
		set_particle_pos(i, initial_pos)
		initial_positions[i] = initial_pos  # Store the initial position
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

# Build the grid for neighbor search
func build_grid():
	grid.clear()
	for i in range(particle_count):
		var pos = multimesh.get_instance_transform_2d(i).origin
		var grid_x = int(pos.x / grid_size)
		var grid_y = int(pos.y / grid_size)
		var cell_key = Vector2(grid_x, grid_y)

		if not grid.has(cell_key):
			grid[cell_key] = []
		grid[cell_key].append(i)

# Perform a grid-based neighbor search with optimization
func update_neighbors():
	for i in range(particle_count):
		neighbors[i] = []  # Reset neighbors list for particle i
	# Iterate only once over grid cells and add symmetrical neighbors
	for cell_key in grid.keys():
		var cell_particles = grid[cell_key]
		for i in range(cell_particles.size()):
			var pi = cell_particles[i]
			var pos_i = multimesh.get_instance_transform_2d(pi).origin
			for j in range(i + 1, cell_particles.size()):
				var pj = cell_particles[j]
				var pos_j = multimesh.get_instance_transform_2d(pj).origin
				if pos_i.distance_to(pos_j) < interaction_radius:
					neighbors[pi].append(pj)
					neighbors[pj].append(pi)
			# Check neighbors in adjacent cells
			for dx in range(-1, 2):
				for dy in range(-1, 2):
					var neighbor_cell = Vector2(cell_key.x + dx, cell_key.y + dy)
					if grid.has(neighbor_cell) and neighbor_cell != cell_key:
						for pj in grid[neighbor_cell]:
							var pos_j = multimesh.get_instance_transform_2d(pj).origin
							if pos_i.distance_to(pos_j) < interaction_radius:
								neighbors[pi].append(pj)
								neighbors[pj].append(pi)

# Cubic Spline Kernel function for density calculations
func cubic_spline_kernel(r: float, h: float) -> float:
	var q = r / h
	if q < 1:
		return (10.0 / (7.0 * PI * h * h)) * (1 - 1.5 * q * q + 0.75 * q * q * q)
	elif q < 2:
		return (10.0 / (7.0 * PI * h * h)) * 0.25 * pow(2 - q, 3)
	return 0.0

# Cubic Spline Kernel gradient
func cubic_spline_gradient(r: float, dx: float, dy: float, h: float) -> Vector2:
	var q = r / h
	if r == 0:
		return Vector2.ZERO
	var grad = Vector2(dx / r, dy / r)
	if q < 1:
		grad *= (10.0 / (7.0 * PI * h * h)) * (-3 * q + 2.25 * q * q)
	elif q < 2:
		grad *= (10.0 / (7.0 * PI * h * h)) * -0.75 * pow(2 - q, 2)
	else:
		grad = Vector2.ZERO
	return grad

# Calculate densities and pressures for each particle
func calculate_density_and_pressure():
	for i in range(particle_count):
		densities[i] = 0.0  # Reset density
		var pos_i = multimesh.get_instance_transform_2d(i).origin

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)

			if distance < interaction_radius:  # Apply kernel
				densities[i] += cubic_spline_kernel(distance, interaction_radius)

		densities[i] = max(densities[i], 0.001)  # Prevent division by zero
		pressures[i] = stiffness * max(densities[i] - rest_density, 0)  # Calculate pressure

# Apply gradient-based pressure forces
func apply_pressure_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		var pressure_force = Vector2.ZERO
		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var dx = pos_j.x - pos_i.x
			var dy = pos_j.y - pos_i.y
			var distance = pos_i.distance_to(pos_j)
			if distance > 0 and distance < interaction_radius:  # Fixed condition
				var grad = cubic_spline_gradient(distance, dx, dy, interaction_radius)
				pressure_force += grad * (pressures[i] + pressures[j]) / (2 * densities[j])
		velocities[i] += pressure_force * delta

# Apply cohesion force for natural clustering
func apply_cohesion_force(delta):
	for i in range(particle_count):
		var cohesion_force = Vector2.ZERO
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance < interaction_radius and distance > 0:
				var direction = (pos_j - pos_i).normalized()
				cohesion_force += direction * (1 - distance / interaction_radius)
		velocities[i] += cohesion_force * cohesion * delta

# Apply surface tension force to enhance droplet behavior
func apply_surface_tension_force(delta):
	for i in range(particle_count):
		var surface_tension_force = Vector2.ZERO
		var pos_i = multimesh.get_instance_transform_2d(i).origin

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)

			if distance < interaction_radius and distance > 0:
				var direction = (pos_j - pos_i).normalized()
				var curvature = (1 - distance / interaction_radius) ** 2  # Curvature factor
				surface_tension_force -= direction * curvature

		velocities[i] += surface_tension_force * surface_tension * delta

# Apply restoring forces to maintain fluid structure
func apply_restoring_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		var restoring_force = (initial_positions[i] - pos_i) * restoring_factor
		velocities[i] += restoring_force * delta

# Apply viscosity forces to each particle
func apply_viscosity_force(delta):
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		var viscosity_force = Vector2.ZERO

		for j in neighbors[i]:
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)

			if distance < interaction_radius and distance > 0:
				var velocity_diff = velocities[j] - velocities[i]
				viscosity_force += velocity_diff * cubic_spline_kernel(distance, interaction_radius)

		velocities[i] += viscosity * viscosity_force * delta

# Handle boundary collisions
func handle_boundary_collision(index: int, pos: Vector2):
	var vel = velocities[index]

	# Compute reflection and damping dynamically based on velocity magnitude
	if pos.x < -boundary_size or pos.x > boundary_size:
		vel.x = -vel.x * velocity_damping  # Reflect along X-axis
		vel *= 1.0 - abs(vel.x / vel.length()) * 0.1  # Reduce energy based on X-impact

	if pos.y < -boundary_size or pos.y > boundary_size:
		vel.y = -vel.y * velocity_damping  # Reflect along Y-axis
		vel *= 1.0 - abs(vel.y / vel.length()) * 0.1  # Reduce energy based on Y-impact

	# Clamp position to stay within bounds
	pos.x = clamp(pos.x, -boundary_size, boundary_size)
	pos.y = clamp(pos.y, -boundary_size, boundary_size)

	velocities[index] = vel
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

# Main simulation loop
func _process(delta):
	build_grid()  # Build the neighbor grid
	update_neighbors()  # Update neighbors for all particles
	calculate_density_and_pressure()  # Compute densities and pressures
	apply_pressure_force(delta)  # Apply gradient-based pressure forces
	apply_viscosity_force(delta)  # Apply viscosity forces
	apply_cohesion_force(delta)  # Apply cohesion forces
	apply_surface_tension_force(delta)  # Apply surface tension forces
	apply_restoring_force(delta)  # Apply restoring forces

	var mouse_pos = get_global_mouse_position()
	apply_mouse_force(mouse_pos, prev_mouse_position)
	prev_mouse_position = mouse_pos

	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		pos_i += velocities[i] * delta
		handle_boundary_collision(i, pos_i)
