using Godot;
using System;

[GlobalClass]
public partial class IsMouseNearby : BTAction
{
	[Export] float range = 400;

    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        if(entity.GetGlobalMousePosition().DistanceTo(entity.GlobalPosition) < range)
		{
			return BTResult.Success;
		}
		return BTResult.Failure;
    }
}
