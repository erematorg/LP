@tool
extends LimbBase
class_name Body

# Array to store Limb references
var limbs: Array[LimbBase] = []

# Array to hold offsets for each limb
@export var offsets: Array[float] = []

# Flag to check if components are initialized
var components_initialized: bool = false

# Threshold to determine if limb position update is needed
@export var position_update_threshold: float = 0.1

func _ready():
	if(limbs.size() < 1):
		return
		
	# Initialize components and update visual representation
	initialize_components()
	update_visual()
	components_initialized = true

	# Collect Limb children and calculate their offsets
	collect_limb_children()
	# Update initial positions of limbs based on the deformed curve
	update_limb_positions()

func _physics_process(_delta):
	# Continuously update positions of limbs during physics processing
	if components_initialized and limbs.size() > 0 and update == true:
		update_limb_positions()

func collect_limb_children():
	# Collect Limb children and calculate initial offsets
	limbs.clear()
	offsets.clear()
	for child in get_children():
		if child is LimbBase:
			limbs.append(child)
			offsets.append(deformed_curve.get_closest_offset(child.position))

func update_limb_positions():
	# Update limb positions to match their offsets on the deformed curve
	if deformed_curve.point_count > 1:
		for i in range(limbs.size()):
			if is_instance_valid(limbs[i]):
				var target_position = deformed_curve.sample_baked(offsets[i])
				if limbs[i].position.distance_to(target_position) > position_update_threshold:
					limbs[i].position = target_position
