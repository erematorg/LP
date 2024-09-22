using Godot;
using System;

public partial class FindFood : BTAction
{
	int tempCounter = 100;

	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		GD.Print("Looking for food... " + tempCounter);
		if(tempCounter <= 0)
		{
			return BTResult.Success;
		}
		tempCounter--;

		return BTResult.Running;
	}
}
