class_name PopulationDynamics extends Node2D

# Constants for the Boids algorithm and spatial partitioning
const VIEW_RADIUS = 50 # Radius within which boids perceive others
const SEPARATION_RADIUS = 20 # Radius within which boids try to separate from others
const COHESION_FACTOR = 0.01 # Factor for cohesion behavior in Boids algorithm
const ALIGNMENT_FACTOR = 0.05 # Factor for alignment behavior in Boids algorithm
const SEPARATION_FACTOR = 0.1 # Factor for separation behavior in Boids algorithm
const GRID_SIZE = 100 # Size of the cells in the spatial grid for optimization

# Exported variables for setting up genetic attributes and carrying capacity
@export var LIST_OF_GENETIC_ATTRIBUTES: Array[GeneticAttributes]
@export var CARRYING_CAPACITY := 150

# Population parameters
var population = [] # List to hold the population entities
var birth_rate = 0.5 # Base birth rate for entities
var death_rate = 0.001 # Base death rate for entities

# Environmental factors (to be integrated with Terrain and Weather Systems)
var resource_availability = 1.0 # Availability of resources in the environment
var weather_impact = 1.0 # Impact of weather on entities

# Genetic attributes template for creating new entities
var genetic_factors = {
	"fertility_rate": 1.0,
	"survival_rate": 1.0,
	"adaptability": 1.0,
	"aggressiveness": 1.0
}

# Spatial grid for optimization to reduce the number of interactions checks
var spatial_grid = {}

# Initialize population
func _ready():
	# Start with a population of 3/4 of the maximum carrying capacity
	for i in range(CARRYING_CAPACITY * 0.75):
		var gene_id: int = randi_range(0, len(LIST_OF_GENETIC_ATTRIBUTES) - 1)
		var gene: GeneticAttributes = LIST_OF_GENETIC_ATTRIBUTES[gene_id]
		gene.sex = randi_range(0, 1) # Assign random sex
		# Create a new entity with random attributes and initial conditions
		var entity = {
			"id": gene_id,
			"age": randi() % gene.span,
			"attributes": {
				"fertility_rate": randf(),
				"survival_rate": randf(),
				"adaptability": randf(),
				"aggressiveness": randf()
			},
			"genetics": gene,
			"position": Vector2(randf() * 1500, randf() * 1000), # Sample initial position
			"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
			"sex": gene.sex
		}
		population.append(entity) # Add entity to the population list
		add_to_grid(entity) # Add entity to the spatial grid
	set_process(true) # Enable processing

# Main processing function
func _process(_delta):
	clear_grid() # Clear the spatial grid
	for entity in population:
		add_to_grid(entity) # Re-add each entity to the spatial grid
	simulate_asexual_births() # Handle asexual reproduction
	simulate_deaths() # Handle deaths
	regulate_population() # Ensure population doesn't exceed carrying capacity
	update_interactions() # Handle interactions between entities
	adapt_to_environment() # Adjust attributes based on environmental factors
	update_positions() # Update positions using the Boids algorithm

# Clear the spatial grid
func clear_grid():
	spatial_grid = {}

# Add an entity to the spatial grid
func add_to_grid(entity):
	var cell = get_cell(entity["position"])
	if not cell in spatial_grid:
		spatial_grid[cell] = []
	spatial_grid[cell].append(entity)

# Get the grid cell for a given position
func get_cell(position):
	return Vector2(floor(position.x / GRID_SIZE), floor(position.y / GRID_SIZE))

# Get neighbors of an entity from the spatial grid
func get_neighbors(entity):
	var neighbors = []
	var cell = get_cell(entity["position"])
	for x in range(cell.x - 1, cell.x + 2):
		for y in range(cell.y - 1, cell.y + 2):
			var neighbor_cell = Vector2(x, y)
			if neighbor_cell in spatial_grid:
				neighbors += spatial_grid[neighbor_cell]
	return neighbors

# Simulate asexual births
func simulate_asexual_births():
	var new_population = []
	for entity in population:
		if (entity["genetics"] as GeneticAttributes).reproduction_type == GeneticAttributes.REPRODUCTION_TYPES.ASEXUAL:
			if entity["age"] >= reproductive_age(entity) and randf() < birth_rate * entity["attributes"]["fertility_rate"]:
				var gene: GeneticAttributes = entity["genetics"]
				# Create new entity with mixed attributes
				var new_entity = {
					"id": entity["id"],
					"age": 0,
					"attributes": mix_attributes(entity["attributes"]),
					"position": entity["position"],
					"genetics": gene,
					"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
					"sex": gene.sex
				}
				new_population.append(new_entity)
				add_to_grid(new_entity)
	population += new_population

# Simulate deaths
func simulate_deaths():
	population = population.filter(func(entity):
		return entity["age"] < entity["genetics"].span and randf() > death_rate * (1.0 - entity["attributes"]["survival_rate"])
	)

# Regulate population to ensure it doesn't exceed carrying capacity
func regulate_population():
	if len(population) > CARRYING_CAPACITY:
		var overpopulation = len(population) - CARRYING_CAPACITY
		for i in range(overpopulation):
			population.pop_back() # Simple removal of excess population

# Calculate reproductive age based on genetic lifespan
func reproductive_age(entity):
	return int(entity["genetics"].span * 0.2) # Example: reproductive age is 20% of lifespan

# Mix attributes of parent entities
func mix_attributes(parent_attributes):
	return {
		"fertility_rate": (parent_attributes["fertility_rate"] + randf()) / 2.0,
		"survival_rate": (parent_attributes["survival_rate"] + randf()) / 2.0,
		"adaptability": (parent_attributes["adaptability"] + randf()) / 2.0,
		"aggressiveness": (parent_attributes["aggressiveness"] + randf()) / 2.0
	}

# Update interactions between entities
func update_interactions():
	for entity in population:
		var neighbors = get_neighbors(entity)
		for other in neighbors:
			if entity != other:
				handle_reproduction(entity, other)
				handle_predation(entity, other)
				handle_competition(entity, other)
				handle_symbiosis(entity, other)

# Handle reproduction between entities
func handle_reproduction(entity, other):
	if entity["genetics"].reproduction_type == GeneticAttributes.REPRODUCTION_TYPES.SEXUAL and other["genetics"].reproduction_type == GeneticAttributes.REPRODUCTION_TYPES.SEXUAL and entity["id"] == other["id"]:
		if entity["age"] >= reproductive_age(entity) and randf() < birth_rate * entity["attributes"]["fertility_rate"]:
			if other["age"] >= reproductive_age(other) and randf() < birth_rate * other["attributes"]["fertility_rate"]:
				if entity["sex"] != other["sex"]:
					var gene: GeneticAttributes = entity["genetics"].merge(other["genetics"])
					gene.sex = randi_range(0, 1)
					var new_entity = {
						"id": entity["id"],
						"age": 0,
						"attributes": mix_attributes(entity["attributes"]),
						"position": entity["position"],
						"genetics": gene,
						"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
						"sex": gene.sex
					}
					population.append(new_entity)
					add_to_grid(new_entity)

# Handle predation interactions
func handle_predation(entity, other):
	if entity["genetics"].trophic_level != other["genetics"].trophic_level:
		if entity["genetics"].trophic_level > other["genetics"].trophic_level:
			other["attributes"]["survival_rate"] -= 0.1 # Entity hunts other
		else:
			entity["attributes"]["survival_rate"] -= 0.1 # Other hunts entity

# Handle competition interactions
func handle_competition(entity, other):
	if entity["genetics"].trophic_level == other["genetics"].trophic_level:
		if entity["attributes"]["aggressiveness"] > other["attributes"]["aggressiveness"]:
			other["attributes"]["survival_rate"] -= 0.1 # Simple competition effect
		else:
			entity["attributes"]["survival_rate"] -= 0.1

# Handle symbiosis interactions
func handle_symbiosis(entity, other):
	if entity["id"] != other["id"] and entity["genetics"].trophic_level == other["genetics"].trophic_level:
		entity["attributes"]["survival_rate"] += 0.1
		other["attributes"]["survival_rate"] += 0.1

# Adapt attributes based on environmental factors
func adapt_to_environment():
	for entity in population:
		entity["attributes"]["adaptability"] *= resource_availability
		entity["attributes"]["survival_rate"] *= weather_impact

# Update positions using Boids algorithm
func update_positions():
	for entity in population:
		var acceleration = Vector2()
		var cohesion = Vector2()
		var alignment = Vector2()
		var separation = Vector2()
		var count = 0
		
		var neighbors = get_neighbors(entity)
		for other in neighbors:
			if entity != other:
				var distance = entity["position"].distance_to(other["position"])
				if distance < VIEW_RADIUS:
					cohesion += other["position"]
					alignment += other["velocity"]
					if distance < SEPARATION_RADIUS:
						separation -= (other["position"] - entity["position"])
					count += 1
		
		if count > 0:
			cohesion = (cohesion / count - entity["position"]) * COHESION_FACTOR
			alignment = (alignment / count - entity["velocity"]) * ALIGNMENT_FACTOR
			separation = separation * SEPARATION_FACTOR
		
		acceleration += cohesion + alignment + separation
		entity["velocity"] += acceleration
		entity["velocity"] = (entity["velocity"] as Vector2).limit_length(entity["genetics"].size / entity["genetics"].weight)
		entity["position"] += entity["velocity"]
