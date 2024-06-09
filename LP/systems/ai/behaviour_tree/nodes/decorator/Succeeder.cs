using Godot;
using System;

[GlobalClass]
public partial class Succeeder : BTDecorator, BTNode //Decorator that always returns Success, regardless of the child node's result.
{
    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        BTNode btNode = GetAsBTNode(GetChild(0));
		BTResult result = btNode.Tick(entity, bb);

		switch(result)
		{
			case BTResult.Running:
				return BTResult.Running;

			default:
				return BTResult.Success;
		}
    }
}