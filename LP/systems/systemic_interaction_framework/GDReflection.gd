class_name GDReflection

# Checks if one property is assignable from another based on class inheritance or type
static func is_assignable_from(property1: Dictionary, property2: Dictionary) -> bool:
	return ClassDB.is_parent_class(property1["class_name"], property2["class_name"]) or \
		((property1.type != TYPE_OBJECT and property2.type != TYPE_OBJECT) and \
		(property1.type == property2.type))

# Caches the exported properties of an object for efficiency
static func get_exported_properties(object: Object) -> Dictionary:
	var result := {}
	for property in object.get_property_list():
		# Only consider script variables and editor-visible properties
		if (property.usage & (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_EDITOR)) == (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_EDITOR):
			result[property.name] = property
	return result

# Transfer values between two objects, using an optional filter for specific properties
static func transfer_values(target1: Object, target2: Object, filter: Array[String] = []):
	var this_properties = GDReflection.get_exported_properties(target1)
	for property in GDReflection.get_exported_properties(target2).values():
		if !this_properties.has(property.name):
			continue
		if !filter.is_empty() and !filter.has(property.name):
			continue
		
		var this_property = this_properties[property.name]
		# Ensure that the property types are compatible before assigning
		if GDReflection.is_assignable_from(this_property, property):
			target1.set(this_property.name, target2.get(property.name))

# Convert an object's properties into a dictionary, optionally using a filter
static func get_property_dict(object: Object, properties: Dictionary, filter: Array[String] = []) -> Dictionary:
	var result := {}
	for property in properties.values():
		if !filter.is_empty() and !filter.has(property.name):
			continue
		result[property.name] = object.get(property.name)
	return result
