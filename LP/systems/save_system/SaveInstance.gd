class_name SaveInstance
extends RigidBody2D

## This is an array of strings that will be converted to nodepaths. Should be used to define properties that should be saved.
@export var instance_save_list: Array[String]

@export var reference: Node

var player_info = "This comes from version 1.0.0"
var pos_test = Vector2(0.4, 0.12)
var array_test = ["This", "is", "a", "string", "array"]
var string = "A string"
var a_variable = 3
var a_dictionary := {
	"a_nested_variable": 4,
}
var new_resource: Resource

# Custom setter and getter for reference
func set_reference(value: Node) -> void:
	reference = value

func get_reference() -> Node:
	return reference

# Custom setter and getter for new_resource
func set_new_resource(value: Resource) -> void:
	new_resource = value

func get_new_resource() -> Resource:
	return new_resource
