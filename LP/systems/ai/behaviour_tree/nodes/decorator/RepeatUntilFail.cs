using Godot;
using System;

[GlobalClass]
public partial class RepeatUntilFail : BTDecorator, BTNode //Decorator that repeats until the child node fails. Returns Success when that happens. Cannot return failure. 
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		BTNode btNode = GetAsBTNode(GetChild(0));
		BTResult btResult = btNode.Tick(entity, bb);

		if(btResult == BTResult.Failure)
		{
			return BTResult.Success;
		}
		else
		{
			return BTResult.Running;
		}
	}
}
