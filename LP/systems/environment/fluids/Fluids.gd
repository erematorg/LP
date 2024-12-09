extends MultiMeshInstance2D

class_name Fluids #Temporary name until separation of concerns for fluids & physics

# --- Editable Parameters ---
@export var particle_count: int = 210  # Number of particles
@export var spawn_size: Vector2 = Vector2(200.0, 100.0)  # Area where particles spawn
@export var smoothing_length: float = 7  # Smoothing length for particle interactions
@export var particle_size: float = 4  # Size of each particle
@export var gravity: Vector2 = Vector2(0, 980)  # Gravity vector (cm/s²)
@export var rest_density: float = 1000.0  # Water's rest density (kg/m³)
@export var surface_tension_coefficient: float = 0.1  # Surface tension coefficient (N/m)
@export var particle_mass: float = 1.0  # Mass per particle (kg)
@export var stiffness_constant: float = 1000  # Stiffness for incompressibility
@export var viscosity_coefficient: float = 0.001  # Dynamic viscosity (Pa·s)
@export var min_velocity: float = 0.5  # Minimum velocity magnitude
@export var grid_cell_size: float = 12  # Size of grid cells for particle sorting
@export var repulsion_strength: float = -1100.0  # Repulsion to avoid particle overlap
@export var spring_constant: float = 50.0  # Spring constant for restoring force
@export var rest_distance: float = 140  # Rest distance for particles

@export var timestep: float = 0.125  # Time step for each simulation frame
@export var boundary_bounce_amount: float = 0.8  # Damping for boundary collisions
@export var velocity_damping: float = 0.99  # Damping of velocity over time

@export var external_objects: Array[Polygon2D] = []  # List of external polygons (polygons entering the fluid)

# --- Non-editable Variables ---
@onready var fluid_instance: MultiMeshInstance2D = self  # Reference to the MultiMeshInstance2D node
@onready var polygon: PackedVector2Array = $Polygon2D.polygon  # Polygon representing the boundary
var avg_velocity: Vector2 = Vector2.ZERO  # Average velocity for freeze state detection
var particles = []  # List holding all particles
var grid: Dictionary = {}  # Spatial grid for neighbor search
var frozen = false  # Flag to indicate if simulation is frozen
var external_polgons: Array[PackedVector2Array]
var smoothing_length_squared = smoothing_length* smoothing_length
var neighbor_counter:float = 0.0
func _ready():
	multimesh = MultiMesh.new()
	_initialize()
func _initialize():
	_detect_multimesh_instance()
	initialize_multimesh()
	initialize_particles()
	apply_shader() 
	smoothing_length_squared = smoothing_length* smoothing_length

func _detect_multimesh_instance():
	if !fluid_instance:
		for child in get_children():
			if child is MultiMeshInstance2D and child.name == "FluidMultiMeshInstance2D":
				fluid_instance = child
			

func initialize_multimesh():
	if fluid_instance.multimesh.instance_count == 0:
		multimesh.transform_format = MultiMesh.TRANSFORM_2D
		multimesh.mesh = create_particle_mesh()
		fluid_instance.multimesh = multimesh
		fluid_instance.multimesh.instance_count = particle_count


func create_particle_mesh() -> Mesh:
	var particle_mesh := QuadMesh.new()
	particle_mesh.size = Vector2(particle_size, particle_size)
	return particle_mesh

func apply_shader():
	var shader = load("res://shaders/Fluids.gdshader") as Shader
	var pmaterial = ShaderMaterial.new()
	pmaterial.shader = shader
	material = pmaterial

func initialize_particles():
	particles.clear()

	for i in range(particle_count):
		var particle = {
			"position": Vector2(randf_range(0, spawn_size.x), randf_range(0, spawn_size.y)),
			"velocity": gravity,
			"force": Vector2.ZERO,
			"density": rest_density,
			"pressure": 0.0,
			"mass": particle_mass,
			"previous_velocity":gravity,
			"neighbors": [0,0,0,0,0,0,0,0]
		}
		particles.append(particle)

func _physics_process(delta):
	
	grid.clear()
	split_clipped_particles()
	assign_particles_to_grid()
	if polygon != $Polygon2D.polygon:
		frozen = false
		
		polygon = $Polygon2D.polygon
	external_polgons.clear()
	for external_object in external_objects:

		external_polgons.append(Transform2D(0,external_object.position)*external_object.polygon)
	calculate_neighbors(delta)
	sph_simulation_step(delta*timestep)
func calculate_neighbors(delta_time):
	neighbor_counter -= delta_time
	if neighbor_counter > 0.0:
		return
	neighbor_counter = delta_time*.5

	for particle in particles:
		particle["neighbors"] = find_neighbors(particle)

func apply_external_forces(particle):
	for i in range(external_polgons.size()):
		
		if Geometry2D.is_point_in_polygon(particle["position"], external_polgons[i]):
			if "solid" in external_objects[i] and external_objects[i].solid == false:
				var direction = particle["position"].direction_to(to_local(external_objects[i].global_position))
				
				particle["force"] = (direction+external_objects[i].velocity.normalized())*external_objects[i].force*100
				continue
			var force = repulsion_strength*100
			
			var nearest_edge = find_nearest_polygon_edge(particle["position"],external_polgons[i])
			var normal = Vector2.ZERO
			if nearest_edge.size() == 2:
				normal = (nearest_edge[1] - nearest_edge[0]).normalized()
				particle["force"] += (normal+external_objects[i].velocity.normalized())*force*100

func sph_simulation_step(delta_time):
	
	avg_velocity = Vector2.ZERO
	for particle in particles:
		
		var neighbors = particle["neighbors"]
		compute_density(particle, neighbors, particle["mass"])
		compute_pressure(particle)
		particle["force"] = 0.01*particle["force"]
		compute_pressure_force(particle, neighbors)
		compute_viscosity_force(particle, neighbors)
		particle["force"] += compute_restoring_force(particle, neighbors)
	
		apply_repulsion_force(particle, neighbors)
		compute_surface_tension_force(particle, neighbors)
		apply_gravity(particle)
		apply_external_forces(particle)
		avg_velocity += particle["velocity"]
	
	avg_velocity /= particle_count
	# Update frozen state based on avg_velocity
	frozen = avg_velocity.length() < 0.15
	for i in range(particles.size()):
		var particle = particles[i]
		
		integrate(particle, delta_time)
		
		handle_boundaries(particle, delta_time)
		if !frozen:
			set_particle_pos(i, particle["position"])

func set_particle_pos(index: int, new_pos: Vector2):
	var local_transform := Transform2D()
	local_transform.origin = new_pos
	multimesh.set_instance_transform_2d(index, local_transform)

func assign_particles_to_grid():
	for particle in particles:
		var cell_x = int(particle["position"].x / grid_cell_size)
		var cell_y = int(particle["position"].y / grid_cell_size)
		var cell_key = Vector2(cell_x, cell_y)
		if not grid.has(cell_key):
			grid[cell_key] = []
		grid[cell_key].append(particle)

func find_neighbors(particle):
	var neighbors = []
	var pos = particle["position"]
	var cell_x = int(pos.x / grid_cell_size)
	var cell_y = int(pos.y / grid_cell_size)

	for dx in [-1, 0, 1]:
		for dy in [-1, 0, 1]:
			var neighbor_cell = Vector2(cell_x + dx, cell_y + dy)
			if grid.has(neighbor_cell):
				for neighbor in grid[neighbor_cell]:
					# Ensure correct distance check
					if pos != neighbor["position"]:  # Compare positions instead of particles
						var dist_sq = pos.distance_squared_to(neighbor["position"])
						if dist_sq < smoothing_length_squared:
							neighbors.append(neighbor)
							# Early exit when limit is reached
							if neighbors.size() >= 8:
								return neighbors
	return neighbors

func compute_density(particle, neighbors, mass):
	var density = kernel_function(0) * mass  # Self-contribution
	for neighbor in neighbors:
		var distance = particle["position"].distance_to(neighbor["position"])
		if distance > 0:
			density += mass * kernel_function(distance)
	particle["density"] = max(density, 10)

func compute_pressure(particle):
	if particle["density"] > rest_density:
		particle["pressure"] = stiffness_constant * ((particle["density"] / rest_density) - 1)
	else:
		particle["pressure"] = 0.0
		
func compute_pressure_force(particle, neighbors):
	var force = Vector2.ZERO
	for neighbor in neighbors:
		var offset = particle["position"] - neighbor["position"]
		var distance = offset.length()
		if distance > 0:
			var direction = offset.normalized()
			var density = max(neighbor["density"], rest_density * 0.9)
			var pressure_term = (particle["pressure"] + neighbor["pressure"]) / (2 * density)
			force += direction * pressure_term * grad_kernel_function(distance)
	particle["force"] += force
	
func compute_viscosity_force(particle, neighbors):
	var force = Vector2.ZERO
	for neighbor in neighbors:
		var velocity_diff = neighbor["velocity"] - particle["velocity"]
		var distance = particle["position"].distance_to(neighbor["position"])
		var density = max(neighbor["density"], rest_density * 0.9)
		force += viscosity_coefficient * neighbor["mass"] * (velocity_diff / density) * laplacian_kernel_function(distance)
	particle["force"] += force

func compute_restoring_force(particle, neighbors):
	var force = Vector2.ZERO
	for neighbor in neighbors:
		var distance = particle["position"].distance_to(neighbor["position"])
		if distance != rest_distance:
			var magnitude = spring_constant * (rest_distance-distance)
			var direction = (particle["position"] - neighbor["position"]).normalized()
			
			
			force += direction * magnitude * velocity_damping
	return force

func apply_repulsion_force(particle, neighbors):
	for neighbor in neighbors:
		var distance = particle["position"].distance_to(neighbor["position"])
		if distance > 0 and distance < smoothing_length:
			distance = max(distance, 0.001)  # Avoid near-zero distances
			var direction = (neighbor["position"] - particle["position"]).normalized()
			var overlap = rest_distance - distance
			var magnitude = repulsion_strength * overlap / (distance + 0.001)
			particle["force"] += direction * magnitude
			
func apply_gravity(particle):
	particle["force"] += gravity * particle["mass"] * particle["density"] / velocity_damping

func integrate(particle, delta_time):
	particle["velocity"] += (particle["force"] / particle["density"]) * delta_time
	particle["velocity"] *= velocity_damping  * (1.0 if !frozen else delta_time)
	 # Enforce minimum velocity

	particle["velocity"] = particle["velocity"].clamp(Vector2(-100,-100),Vector2(100,100))
	
	var movement = particle["velocity"] * delta_time
	
	
	particle["position"] += movement

func kernel_function(distance):
	var q = distance / smoothing_length
	if q < 1:
		return 15 / (7 * PI * pow(smoothing_length, 2)) * pow(1 - q, 2)
	elif q < 2:
		return 15 / (7 * PI * pow(smoothing_length, 2)) * pow(2 - q, 2)
	else:
		return 0

func grad_kernel_function(distance):
	var q = distance / smoothing_length
	if q < 1:
		return -45 / (PI * pow(smoothing_length, 6)) * pow(1 - q, 2)
	elif q < 2:
		return -45 / (PI * pow(smoothing_length, 6)) * (2 - q)
	else:
		return 0

func laplacian_kernel_function(distance):
	var q = distance / smoothing_length
	if q < 1:
		return 45 / (PI * pow(smoothing_length, 6)) * (1 - q)
	elif q < 2:
		return 45 / (PI * pow(smoothing_length, 6)) * (2 - q)
	else:
		return 0

func compute_surface_curvature(particle, neighbors):
	var curvature = 0.0
	for neighbor in neighbors:
		var distance = particle["position"].distance_to(neighbor["position"])
		if distance > 0:
			curvature += kernel_function(distance) * neighbor["mass"] / neighbor["density"]
	return curvature

func is_surface_particle(particle, neighbors):
	var gradient = compute_color_field_gradient(particle, neighbors)
	return gradient.length() > 0.1  

func compute_surface_tension_force(particle, neighbors):
	var gradient = compute_color_field_gradient(particle, neighbors)
	if gradient.length() > 0.01:  # Threshold for normalization
		var curvature = compute_surface_curvature(particle, neighbors)
		var force = -surface_tension_coefficient * curvature * gradient.normalized()
		particle["force"] += force

func compute_color_field_gradient(particle, neighbors):
	var gradient = Vector2.ZERO
	for neighbor in neighbors:
		var distance = particle["position"].distance_to(neighbor["position"])
		if distance > 0:
			var direction = (neighbor["position"] - particle["position"]).normalized()
			gradient += direction * neighbor["mass"] / neighbor["density"] * grad_kernel_function(distance)
	return gradient

func compute_boundary_force(particle):
	var predicted_pos: Vector2 = particle["position"]
	if not Geometry2D.is_point_in_polygon(predicted_pos, polygon):
		var nearest_edge = find_nearest_polygon_edge(predicted_pos,polygon)
		if nearest_edge.size() == 2:
			var normal = (nearest_edge[1] - nearest_edge[0]).normalized().orthogonal()
			var overlap = Geometry2D.get_closest_point_to_segment(predicted_pos, nearest_edge[0], nearest_edge[1]) - predicted_pos
			return normal * (repulsion_strength * max(0.0, smoothing_length - overlap.length()))
	return Vector2.ZERO

func handle_boundaries(particle, delta_time):
	
	var predicted_pos: Vector2 = particle["position"] + particle["velocity"] * delta_time

	# Check if the particle is outside the polygon
	
	for polyindex in range(external_objects.size()):
		if "solid" in external_objects[polyindex] and !external_objects[polyindex].solid:
			continue
		if Geometry2D.is_point_in_polygon(particle["position"],external_polgons[polyindex]):
			var nearest_edge = find_nearest_polygon_edge(particle["position"],external_polgons[polyindex])
			if nearest_edge.size() == 2:
				var closest_point = Geometry2D.get_closest_point_to_segment(particle["position"], nearest_edge[0], nearest_edge[1])
				particle["position"] = closest_point
	if not Geometry2D.is_point_in_polygon(predicted_pos, polygon):
		# Handle collision normally
		var nearest_edge = find_nearest_polygon_edge(predicted_pos,polygon)
		if nearest_edge.size() == 2:
			var normal = (nearest_edge[1] - nearest_edge[0]).normalized().orthogonal()
			var dot = particle["velocity"].dot(normal) 
			var tangent = particle["velocity"] - dot * normal *0.4
			var tangential_force = Vector2(normal.y, 0) * randf_range(-0.5, 0.5) * repulsion_strength * 0.1
			# Reflect velocity and slightly dampen
			particle["velocity"] = tangent + normal*dot * -boundary_bounce_amount + tangential_force

			# Snap particle back inside
			var closest_point = Geometry2D.get_closest_point_to_segment(predicted_pos, nearest_edge[0], nearest_edge[1])
			particle["position"] = closest_point 
			
	
func find_nearest_polygon_edge(point: Vector2,poly) -> Array:
	var nearest_edge: Array = []
	var min_distance = INF
	
	for i in range(polygon.size()):
		var start_point = poly[i% poly.size()]
		var end_point = poly[(i + 1) % poly.size()]  # Wrap around for the last edge
		var closest_point = Geometry2D.get_closest_point_to_segment(point, start_point, end_point)
		var distance = closest_point.distance_to(point)
		
		if distance < min_distance:
			min_distance = distance
			nearest_edge = [start_point, end_point]
	
	return nearest_edge
func split_clipped_particles():
	var threshold_sq = (particle_size * 2) * (particle_size * 2)  # Precompute the squared threshold
	for cell_particles in grid.values():
		var count = cell_particles.size()
		for i in range(count):
			var particle_a = cell_particles[i]
			for j in range(i + 1, count):
				var particle_b = cell_particles[j]
				var dist_sq = particle_a["position"].distance_squared_to(particle_b["position"])
				if dist_sq < threshold_sq:
					# Calculate actual distance only when needed
					resolve_clipping(particle_a, particle_b, sqrt(dist_sq))

func resolve_clipping(particle_a, particle_b, distance):
	if distance > 0:  # Avoid division by zero
		var direction = (particle_b["position"] - particle_a["position"]).normalized()
		var overlap = particle_size * 2- distance
		var separation = direction * overlap
		# Distribute separation
		particle_a["position"] -= separation
		particle_b["position"] += separation
