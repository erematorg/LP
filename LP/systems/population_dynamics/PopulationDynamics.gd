class_name PopulationDynamics extends Node2D

const VIEW_RADIUS = 50 # Boids algorithm
const SEPARATION_RADIUS = 20 # Boids algorithm
const COHESION_FACTOR = 0.01 # Boids algorithm
const ALIGNMENT_FACTOR = 0.05 # Boids algorithm
const SEPARATION_FACTOR = 0.1 # Boids algorithm

@export var LIST_OF_GENETIC_ATTRIBUTES: Array[GeneticAttributes]
@export var CARRYING_CAPACITY := 150

# Population parameters
var population = []
var birth_rate = 0.5
var death_rate = 0.001

# Environmental factors (to be integrated with Terrain and Weather Systems)
var resource_availability = 1.0
var weather_impact = 1.0

# Genetic attributes (sample structure)
var genetic_factors = {
	"fertility_rate": 1.0,
	"survival_rate": 1.0,
	"adaptability": 1.0,
	"aggressiveness": 1.0
}

# Initialize population
func _ready():
	for i in range(CARRYING_CAPACITY * 0.75):  # starting with a population of 3/4 of the max capacity
		var gene_id: int = randi_range(0, len(LIST_OF_GENETIC_ATTRIBUTES) - 1)
		var gene: GeneticAttributes = LIST_OF_GENETIC_ATTRIBUTES[gene_id]
		gene.sex = randi_range(0, 1)
		population.append({
			"id": gene_id,
			"age": randi() % gene.span,
			"attributes": {
				"fertility_rate": randf(),
				"survival_rate": randf(),
				"adaptability": randf(),
				"aggressiveness": randf()
			},
			"genetics": gene,
			"position": Vector2(randf() * 1500, randf() * 1000),  # sample position
			"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
			"sex": gene.sex
		})
	set_process(true)

func _process(_delta):
	simulate_asexual_births()
	simulate_deaths()
	regulate_population()
	update_interactions() # competition, predation, reproduction, symbiosis
	adapt_to_environment()
	update_positions()

func simulate_asexual_births():
	var new_population = []
	for entity in population:
		if (entity["genetics"] as GeneticAttributes).reproduction_type == 1: # asexual reproduction
			if entity["age"] >= reproductive_age(entity) and randf() < birth_rate * entity["attributes"]["fertility_rate"]:
				var gene: GeneticAttributes = entity["genetics"]
				new_population.append({
					"id": entity["id"],
					"age": 0,
					"attributes": mix_attributes(entity["attributes"]),
					"position": entity["position"],
					"genetics": gene,
					"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
					"sex": gene.sex
				})
	population += new_population

func simulate_deaths():
	population = population.filter(func(entity):
		return entity["age"] < entity["genetics"].span and randf() > death_rate * (1.0 - entity["attributes"]["survival_rate"])
	)

func regulate_population():
	if len(population) > CARRYING_CAPACITY:
		var overpopulation = len(population) - CARRYING_CAPACITY
		for i in range(overpopulation):
			population.pop_back()  # simple removal, can be replaced with a more sophisticated method

func reproductive_age(entity):
	return int(entity["genetics"].span * 0.2)  # example reproductive age is 20% of MAX_AGE

func mix_attributes(parent_attributes):
	# custom genetic mixing (ideally should be done in GeneticAttributes, but I don't want to modify that file too much :p)
	return {
		"fertility_rate": (parent_attributes["fertility_rate"] + randf()) / 2.0,
		"survival_rate": (parent_attributes["survival_rate"] + randf()) / 2.0,
		"adaptability": (parent_attributes["adaptability"] + randf()) / 2.0,
		"aggressiveness": (parent_attributes["aggressiveness"] + randf()) / 2.0
	}

func update_interactions():
	for entity in population:
		for other in population:
			if entity != other:
				interact(entity, other)

func interact(entity, other):
	var distance = entity["position"].distance_to(other["position"])
	if distance < VIEW_RADIUS:
		# sexual reproduction
		if entity["genetics"].reproduction_type == 0 and other["genetics"].reproduction_type == 0 and entity["id"] == other["id"]:
			if entity["age"] >= reproductive_age(entity) and randf() < birth_rate * entity["attributes"]["fertility_rate"]:
				if other["age"] >= reproductive_age(other) and randf() < birth_rate * other["attributes"]["fertility_rate"]:
					if entity["sex"] != other["sex"]:
						var gene: GeneticAttributes = entity["genetics"].merge(other["genetics"])
						gene.sex = randi_range(0, 1)
						population.append({
							"id": entity["id"],
							"age": 0,
							"attributes": mix_attributes(entity["attributes"]),
							"position": entity["position"],
							"genetics": gene,
							"velocity": Vector2(randf(), randf()) * gene.size / gene.weight,
							"sex": gene.sex
						})
		
		# predation (for entities with different trophic level)
		if entity["genetics"].trophic_level != other["genetics"].trophic_level:
			if entity["genetics"].trophic_level > other["genetics"].trophic_level:
				other["attributes"]["survival_rate"] -= 0.1 # entity hunts other
			else:
				entity["attributes"]["survival_rate"] -= 0.1 # other hunts entity
		
		# competition (for entities with same trophic level, can consist of different genes to account for both interspecific and intraspecific competition)
		else:
			if entity["attributes"]["aggressiveness"] > other["attributes"]["aggressiveness"]:
				other["attributes"]["survival_rate"] -= 0.1  # simple competition effect
			else:
				entity["attributes"]["survival_rate"] -= 0.1
		
		# symbiosis (for entities with same trophic level and different gene)
		if entity["id"] != other["id"] and entity["genetics"].trophic_level == other["genetics"].trophic_level:
			entity["attributes"]["survival_rate"] += 0.1
			other["attributes"]["survival_rate"] += 0.1
		

func adapt_to_environment():
	for entity in population:
		# Adjust attributes based on environmental factors
		entity["attributes"]["adaptability"] *= resource_availability
		entity["attributes"]["survival_rate"] *= weather_impact

func update_positions(): # currently only Boids Algorithm is implemented (which is suitable for movable entities like Animalia but not suitable for static entities like Plantae)
	for entity in population:
		var acceleration = Vector2()
		var cohesion = Vector2()
		var alignment = Vector2()
		var separation = Vector2()
		var count = 0
		
		for other in population:
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
