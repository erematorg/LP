extends Node2D

var fluid_instance: MultiMeshInstance2D
var multimesh := MultiMesh.new()

@export var particle_count := 50
@export var boundary_size := 250
@export var particle_size := 6.0
@export var interaction_radius := 10.0
@export var rest_density := 1.5
@export var stiffness := 200.0  # Pressure stiffness
@export var viscosity := 0.1   # Viscosity factor
@export var velocity_damping := 0.97  # Velocity reduction upon collision

# Particle properties
var velocities = []
var neighbors = []
var densities = []
var pressures = []

func _ready():
	_detect_multimesh_instance()
	initialize_multimesh()
	initialize_particles()

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

# Initialize particle positions, velocities, neighbors, densities, and pressures
func initialize_particles():
	velocities.resize(particle_count)
	neighbors.resize(particle_count)
	densities.resize(particle_count)
	pressures.resize(particle_count)

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

# Perform a basic neighbor search
func update_neighbors():
	for i in range(particle_count):
		neighbors[i] = []  # Reset neighbors list for particle i
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		for j in range(particle_count):
			if i == j:
				continue
			var pos_j = multimesh.get_instance_transform_2d(j).origin
			var distance = pos_i.distance_to(pos_j)
			if distance < interaction_radius:
				neighbors[i].append(j)

# Calculate densities and pressures for each particle
func calculate_density_and_pressure():
	for i in range(particle_count):
		var density = 0.0
		for j in neighbors[i]:
			var pos_i = multimesh.get_instance_transform_2d(i).origin
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
	# Reflect velocity and apply damping if particle hits a boundary
	if pos.x < -boundary_size or pos.x > boundary_size:
		velocities[index].x = -velocities[index].x * velocity_damping
	if pos.y < -boundary_size or pos.y > boundary_size:
		velocities[index].y = -velocities[index].y * velocity_damping

	# Clamp position to stay within the boundary
	pos.x = clamp(pos.x, -boundary_size, boundary_size)
	pos.y = clamp(pos.y, -boundary_size, boundary_size)
	set_particle_pos(index, pos)

# Main simulation loop
func _process(delta):
	update_neighbors()  # Update neighbors for all particles
	calculate_density_and_pressure()  # Compute densities and pressures
	apply_pressure_force(delta)  # Apply pressure forces
	apply_viscosity_force(delta)  # Apply viscosity forces

	# Update particle positions and handle boundary collisions
	for i in range(particle_count):
		var pos_i = multimesh.get_instance_transform_2d(i).origin
		pos_i += velocities[i] * delta
		handle_boundary_collision(i, pos_i)
