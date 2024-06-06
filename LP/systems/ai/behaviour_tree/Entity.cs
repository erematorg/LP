using Godot;
using System;

public partial class Entity : Node2D
{
	public void MoveToPosition(Vector2 position)
	{
		GlobalPosition = GlobalPosition.MoveToward(position, 4.0f);
	}
}
