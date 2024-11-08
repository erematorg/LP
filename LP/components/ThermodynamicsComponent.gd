extends Node
class_name ThermodynamicsComponent

# Thermal properties and constants
@export var cooling_rate := 0.2
@export var diffusion_rate := 0.15
@export var max_temperature := 100
@export var min_temperature := 0
@export var convection_rate := 0.05
@export var inertia := 0.9
@export var ambient_temperature := 25

# Externally attached grids
var temperature_grid : Array = []
var heat_capacity_grid : Array = []

# Heat sources dictionary
var heat_sources : Dictionary = {}

### Grid Management
# Attach an external grid
func set_grid(temp_grid: Array, capacity_grid: Array):
	temperature_grid = temp_grid
	heat_capacity_grid = capacity_grid

### Heat Source Management
# Add a heat source with initial energy
func add_heat_source(row: int, col: int, intensity: float, energy: float):
	if is_within_bounds(row, col):
		heat_sources[[row, col]] = { "intensity": intensity, "energy": energy }
		print("Heat source added at row:", row, "col:", col)

# Apply heat from sources to the grid
func apply_heat_sources():
	for position in heat_sources.keys():
		var row = position[0]
		var col = position[1]
		var source_data = heat_sources[position]
		var intensity = source_data["intensity"]
		var energy = source_data["energy"]

		# Apply heat based on heat capacity and intensity
		heat_capacity_grid[row][col] = calc_heat_capacity(temperature_grid[row][col])
		temperature_grid[row][col] += intensity / heat_capacity_grid[row][col]

		# Exponential decay for the energy of heat sources
		heat_sources[position]["energy"] *= 0.98  # Decay factor of 2%
		if heat_sources[position]["energy"] <= 0.1:
			heat_sources.erase(position)

### Radiative Cooling
# Cool grid cells based on radiative cooling
func apply_radiative_cooling():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var temp = temperature_grid[row][col]
			var heat_capacity = heat_capacity_grid[row][col]

			# Adjust radiative cooling rate based on temperature
			var cooling_factor = cooling_rate * (1 + (temp / max_temperature) * 0.5)
			temperature_grid[row][col] -= cooling_factor / heat_capacity
			temperature_grid[row][col] = max(temperature_grid[row][col], min_temperature)

### Thermal Conduction
# Distribute heat between neighboring cells for conduction
func apply_thermal_conduction():
	var new_temp_grid = temperature_grid.duplicate(true)
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var current_temp = temperature_grid[row][col]
			var temp_gradient_sum = 0.0
			for neighbor_temp in get_neighbors(row, col):
				var temp_diff = neighbor_temp - current_temp

				# Increase conduction rate for larger temperature differences
				var dynamic_diffusion = diffusion_rate * (1.0 + abs(temp_diff) / max_temperature)
				temp_gradient_sum += dynamic_diffusion * temp_diff

			# Calculate updated temperature with conductivity adjustment
			heat_capacity_grid[row][col] = calc_heat_capacity(current_temp)
			new_temp_grid[row][col] += temp_gradient_sum / heat_capacity_grid[row][col]

	temperature_grid = new_temp_grid

### Convection
# Move heat vertically and horizontally within the grid
func apply_convection():
	var new_temp_grid = temperature_grid.duplicate(true)
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var convection_sum = 0.0
			if row > 0:
				convection_sum += (temperature_grid[row - 1][col] - temperature_grid[row][col]) * convection_rate
			if row < temperature_grid.size() - 1:
				convection_sum += (temperature_grid[row + 1][col] - temperature_grid[row][col]) * convection_rate
			if col > 0:
				convection_sum += (temperature_grid[row][col - 1] - temperature_grid[row][col]) * convection_rate
			if col < temperature_grid[row].size() - 1:
				convection_sum += (temperature_grid[row][col + 1] - temperature_grid[row][col]) * convection_rate

			heat_capacity_grid[row][col] = calc_heat_capacity(temperature_grid[row][col])
			new_temp_grid[row][col] += convection_sum / heat_capacity_grid[row][col]

	temperature_grid = new_temp_grid

### Inertia
# Gradually apply thermal inertia for smooth temperature changes
func apply_inertia():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			temperature_grid[row][col] = (
				temperature_grid[row][col] * inertia +
				temperature_grid[row][col] * (1 - inertia)
			)

### Boundary Conditions
# Enforce cooling at grid edges to simulate interaction with ambient environment
func enforce_boundary_conditions():
	var edge_insulation = 0.5
	for col in range(temperature_grid[0].size()):
		temperature_grid[0][col] -= cooling_rate * edge_insulation * (temperature_grid[0][col] - ambient_temperature) / max_temperature
		temperature_grid[temperature_grid.size() - 1][col] -= cooling_rate * edge_insulation * (temperature_grid[temperature_grid.size() - 1][col] - ambient_temperature) / max_temperature
	for row in range(temperature_grid.size()):
		temperature_grid[row][0] -= cooling_rate * edge_insulation * (temperature_grid[row][0] - ambient_temperature) / max_temperature
		temperature_grid[row][temperature_grid[row].size() - 1] -= cooling_rate * edge_insulation * (temperature_grid[row][temperature_grid[row].size() - 1] - ambient_temperature) / max_temperature

# Clamp temperature values to stay within defined min and max
func clamp_temperature_bounds():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			temperature_grid[row][col] = clamp(temperature_grid[row][col], min_temperature, max_temperature)

### Utility Functions
# Calculate dynamic heat capacity based on current temperature
func calc_heat_capacity(temperature: float) -> float:
	return 1.0 + 0.5 / (1.0 + exp(-(temperature - 50) * 0.1))

# Calculate dynamic thermal conductivity based on temperature
func calc_conductivity(temperature: float) -> float:
	return 0.1 * (1.0 + 0.01 * temperature / max_temperature)

# Get neighboring temperatures for conduction and convection
func get_neighbors(row: int, col: int) -> Array:
	var neighbors = []
	if row > 0:
		neighbors.append(temperature_grid[row - 1][col])
	if row < temperature_grid.size() - 1:
		neighbors.append(temperature_grid[row + 1][col])
	if col > 0:
		neighbors.append(temperature_grid[row][col - 1])
	if col < temperature_grid[row].size() - 1:
		neighbors.append(temperature_grid[row][col + 1])
	return neighbors

# Check if cell is within grid bounds
func is_within_bounds(row: int, col: int) -> bool:
	return row >= 0 and row < temperature_grid.size() and col >= 0 and col < temperature_grid[row].size()

### Update thermal state by applying all processes
func update_state():
	apply_heat_sources()
	apply_radiative_cooling()
	apply_thermal_conduction()
	apply_convection()
	apply_inertia()
	enforce_boundary_conditions()
	clamp_temperature_bounds()
