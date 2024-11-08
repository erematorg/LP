extends Node
class_name DNAComponent
##Temporary File only for testing purposes will later be way more enhanced for the best DNA system out there.
var genetic_attributes: Dictionary

func initialize(parent1: Attributes, parent2: Attributes):
	genetic_attributes = {}
	for key in parent1.keys():
		genetic_attributes[key] = (parent1[key] + parent2[key]) / 2
	introduce_variation()

func introduce_variation():
	for key in genetic_attributes.keys():
		genetic_attributes[key] *= randf_range(0.95, 1.05)

func get_attribute(attribute: String):
	return genetic_attributes.get(attribute, null)
