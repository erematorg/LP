using Godot;
using System;

[GlobalClass]
public partial class Inverter : BTDecorator, BTNode //Returns the opposite of the child node's result.
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		BTNode btNode = GetAsBTNode(GetChild(0));
		BTResult result = btNode.Tick(entity, bb);

		switch(result)
		{
			case BTResult.Running:
				return BTResult.Running;
			case BTResult.Success:
				return BTResult.Failure;
			case BTResult.Failure:
				return BTResult.Success;

			//To keep the compiler happy :) This will (ideally) never be hit.
			default:
				return BTResult.Running;
		}
	}
}
