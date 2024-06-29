class_name GDReflection

static func is_assignable_from(property1: Dictionary, property2: Dictionary) -> bool:
	return ClassDB.is_parent_class(property1["class_name"], property2["class_name"]) or ((property1.type != TYPE_OBJECT and property2.type != TYPE_OBJECT) and (property1.type == property2.type))

static func get_exported_properties(object: Object) -> Dictionary:
	var result := {}
	for property in object.get_property_list():
		if (property.usage & (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_EDITOR)) == (PROPERTY_USAGE_SCRIPT_VARIABLE | PROPERTY_USAGE_EDITOR):
			result[property.name] = property
	return result

static func transfer_values(target1: Object, target2: Object, filter: Array[String] = []) -> void:
	var this_properties := GDReflection.get_exported_properties(target1)
	for property in GDReflection.get_exported_properties(target2).values():
		if !this_properties.has(property.name): continue
		if !filter.is_empty() and !filter.has(property.name): continue
		
		var this_property: Dictionary = this_properties[property.name]
		if GDReflection.is_assignable_from(this_property, property):
			target1.set(this_property.name, target2.get(property.name))

static func get_property_dict(object: Object, properties: Dictionary, filter: Array[String] = []) -> Dictionary:
	var result := {}
	for property in properties.values():
		if !filter.is_empty() and !filter.has(property.name): continue
		result[property.name] = object.get(property)
	return result
