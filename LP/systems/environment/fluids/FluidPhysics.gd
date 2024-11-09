extends Node2D

var fluid_instance: MultiMeshInstance2D
var multimesh := MultiMesh.new()

@export var particle_count := 50
@export var boundary_size := 250
@export var particle_size := 6.0

# Particle properties
var velocities = []

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

# Initialize particle positions and velocities
func initialize_particles():
	velocities.resize(particle_count)

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
