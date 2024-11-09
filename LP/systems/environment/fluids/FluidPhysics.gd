extends Node2D

var fluid_instance: MultiMeshInstance2D
var multimesh := MultiMesh.new()

@export var particle_count := 50
@export var boundary_size := 250
@export var particle_size := 6.0
@export var interaction_radius := 10.0  # Radius for neighbor detection

# Particle properties
var velocities = []
var neighbors = []

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

# Initialize particle positions, velocities, and neighbors list
func initialize_particles():
	velocities.resize(particle_count)
	neighbors.resize(particle_count)

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
				neighbors[i].append(j)  # Add j to i's neighbor list

# Debug neighbors for a specific particle (optional)
func debug_neighbors(particle_index: int):
	print("Neighbors of particle ", particle_index, ": ", neighbors[particle_index])

# Main simulation loop
func _process(delta):
	update_neighbors()  # Update neighbors for all particles
