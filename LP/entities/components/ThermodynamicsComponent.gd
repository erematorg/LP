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
@export var expansion_coefficient := 0.001  # Coefficient for density change
@export var base_density := 1.0  # Density at ambient temperature
@export var pressure_coefficient := 1.0  # Proportional constant for pressure calculation

# Externally attached grids
var temperature_grid : Array = []
var heat_capacity_grid : Array = []
var density_grid : Array = []  # Dynamic densities
var pressure_grid : Array = []

# Heat sources dictionary
var heat_sources : Dictionary = {}

### Grid Management
# Attach an external grid
func set_grid(temp_grid: Array, capacity_grid: Array):
	temperature_grid = temp_grid
	heat_capacity_grid = capacity_grid
	density_grid = []
	pressure_grid = []
	for row in range(temp_grid.size()):
		var row_density = []
		var row_pressure = []
		for col in range(temp_grid[row].size()):
			row_density.append(calc_density(temp_grid[row][col]))
			row_pressure.append(0.0)  # Initialize pressure as 0
		density_grid.append(row_density)
		pressure_grid.append(row_pressure)

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

		# Apply heat to the current cell
		heat_capacity_grid[row][col] = calc_heat_capacity(temperature_grid[row][col])
		temperature_grid[row][col] += intensity / heat_capacity_grid[row][col]

		# Distribute heat to neighboring cells
		for neighbor_index in range(get_neighbors(row, col).size()):
			var neighbor_temp = get_neighbors(row, col)[neighbor_index]
			
			# Determine neighbor position
			var n_row = row
			var n_col = col
			if neighbor_index == 0 and row > 0:  # Top neighbor
				n_row = row - 1
			elif neighbor_index == 1 and row < temperature_grid.size() - 1:  # Bottom neighbor
				n_row = row + 1
			elif neighbor_index == 2 and col > 0:  # Left neighbor
				n_col = col - 1
			elif neighbor_index == 3 and col < temperature_grid[row].size() - 1:  # Right neighbor
				n_col = col + 1

			# Apply heat to the neighbor
			if is_within_bounds(n_row, n_col):
				heat_capacity_grid[n_row][n_col] = calc_heat_capacity(temperature_grid[n_row][n_col])
				temperature_grid[n_row][n_col] += (intensity * 0.1) / heat_capacity_grid[n_row][n_col]  # Spread 10% of intensity

		# Exponential decay for energy
		heat_sources[position]["energy"] *= 0.98
		if heat_sources[position]["energy"] <= 0.1:
			heat_sources.erase(position)

### Radiative Cooling
func apply_radiative_cooling():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var temp = temperature_grid[row][col]
			var heat_capacity = heat_capacity_grid[row][col]
			var dynamic_cooling_rate = cooling_rate * (1.0 + (temp / max_temperature)**2)
			temperature_grid[row][col] -= dynamic_cooling_rate / heat_capacity
			temperature_grid[row][col] = max(temperature_grid[row][col], min_temperature)

### Thermal Conduction
func apply_thermal_conduction():
	var new_temp_grid = temperature_grid.duplicate(true)
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var current_temp = temperature_grid[row][col]
			var temp_gradient_sum = 0.0
			for neighbor_temp in get_neighbors(row, col):
				var temp_diff = neighbor_temp - current_temp
				var dynamic_diffusion = diffusion_rate * (1.0 + abs(temp_diff) / max_temperature)
				temp_gradient_sum += dynamic_diffusion * temp_diff

			heat_capacity_grid[row][col] = calc_heat_capacity(current_temp)
			new_temp_grid[row][col] += temp_gradient_sum / heat_capacity_grid[row][col]

	temperature_grid = new_temp_grid

### Convection with Pressure Dynamics
func apply_convection():
	var new_temp_grid = temperature_grid.duplicate(true)
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var convection_sum = 0.0

			# Pressure-driven convection
			if row > 0:
				var pressure_diff = pressure_grid[row - 1][col] - pressure_grid[row][col]
				convection_sum += convection_rate * pressure_diff
			if row < temperature_grid.size() - 1:
				var pressure_diff = pressure_grid[row + 1][col] - pressure_grid[row][col]
				convection_sum += convection_rate * pressure_diff
			if col > 0:
				var pressure_diff = pressure_grid[row][col - 1] - pressure_grid[row][col]
				convection_sum += convection_rate * pressure_diff
			if col < temperature_grid[row].size() - 1:
				var pressure_diff = pressure_grid[row][col + 1] - pressure_grid[row][col]
				convection_sum += convection_rate * pressure_diff

			var turbulence = randf() * convection_rate * 0.1
			convection_sum += turbulence * (randf() - 0.5)

			heat_capacity_grid[row][col] = calc_heat_capacity(temperature_grid[row][col])
			new_temp_grid[row][col] += convection_sum / heat_capacity_grid[row][col]

	temperature_grid = new_temp_grid

### Pressure Calculation
func calc_pressure(temperature: float, density: float) -> float:
	return pressure_coefficient * density * temperature

### Update Pressures
func update_pressures():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			pressure_grid[row][col] = calc_pressure(temperature_grid[row][col], density_grid[row][col])

### Inertia
func apply_inertia():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			temperature_grid[row][col] = (
				temperature_grid[row][col] * inertia +
				temperature_grid[row][col] * (1 - inertia)
			)

### Boundary Conditions
func enforce_boundary_conditions():
	var edge_insulation = 0.5
	for col in range(temperature_grid[0].size()):
		var gradient_top = abs(temperature_grid[0][col] - temperature_grid[1][col])
		temperature_grid[0][col] -= cooling_rate * edge_insulation * gradient_top / max_temperature
		var gradient_bottom = abs(temperature_grid[temperature_grid.size() - 1][col] - temperature_grid[temperature_grid.size() - 2][col])
		temperature_grid[temperature_grid.size() - 1][col] -= cooling_rate * edge_insulation * gradient_bottom / max_temperature

	for row in range(temperature_grid.size()):
		var gradient_left = abs(temperature_grid[row][0] - temperature_grid[row][1])
		temperature_grid[row][0] -= cooling_rate * edge_insulation * gradient_left / max_temperature
		var gradient_right = abs(temperature_grid[row][temperature_grid[row].size() - 1] - temperature_grid[row][temperature_grid[row].size() - 2])
		temperature_grid[row][temperature_grid[row].size() - 1] -= cooling_rate * edge_insulation * gradient_right / max_temperature

### Temperature Clamping
func clamp_temperature_bounds():
	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			var temp = temperature_grid[row][col]
			if temp < min_temperature:
				temperature_grid[row][col] = min_temperature
			elif temp > max_temperature:
				temperature_grid[row][col] = max_temperature

### Utility Functions
func calc_heat_capacity(temperature: float) -> float:
	return 1.0 + 0.5 * (1.0 - (temperature / max_temperature))

func calc_density(temperature: float) -> float:
	return base_density * (1.0 - expansion_coefficient * (temperature - ambient_temperature))

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

func is_within_bounds(row: int, col: int) -> bool:
	return row >= 0 and row < temperature_grid.size() and col >= 0 and col < temperature_grid[row].size()

### Update State
func update_state():
	apply_heat_sources()
	apply_radiative_cooling()
	apply_thermal_conduction()
	update_pressures()
	apply_convection()
	apply_inertia()
	enforce_boundary_conditions()
	clamp_temperature_bounds()

	for row in range(temperature_grid.size()):
		for col in range(temperature_grid[row].size()):
			density_grid[row][col] = calc_density(temperature_grid[row][col])
