using Godot;
using System;

[GlobalClass]
public abstract partial class BTAction : Node, BTNode
{
	public override void _Ready()
	{
		if(GetChildCount() != 0)
		{
			GD.PrintErr($"Leaf node {Name} must have no child nodes!");
		}
	}

	public abstract BTResult Tick(Entity entity, Blackboard bb);
}
