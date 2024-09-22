using Godot;
using System;

[GlobalClass]
public partial class PrintSeperator : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		GD.Print("--------------------------------------------------");
		return BTResult.Success;
	}
}
