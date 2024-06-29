@tool class_name NonLifeAttributes extends GeneticAttributes

## Specifies the temperature range in which the element exists.
@export var temperature_range: Vector2

## Indicates the element's conductivity, affecting its interaction with heat.
@export var conductivity: float

## Defines the element's density, influencing its buoyancy or gravitational effects.
@export var density: float

## Specifies the element's opacity, affecting its transparency or visibility.
@export var opacity: float

## Indicates the element's melting point, the temperature at which it changes from solid to liquid.
@export var melting_point: float

## Indicates the element's boiling point, the temperature at which it changes from liquid to gas.
@export var boiling_point: float

## Specifies the element's malleability, affecting its ability to be hammered or pressed into shapes without breaking.
@export var malleability: float

## Indicates the element's flammability, affecting its ability to ignite and burn.
@export var flammability: float

## Indicates the element's buoyancy, affecting its ability to float.
@export var buoyancy: float

## Indicates the element's magnetism, affecting its ability to attract or repel other materials based on their magnetic fields.
@export var magnetism: float

## Indicates the element's radioactivity, affecting its emission of radiation from unstable atomic nuclei.
@export var radioactivity: float

## Specifies whether the element emits light.
@export var emits_light: bool
