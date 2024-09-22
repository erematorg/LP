using Godot;
using System;

[GlobalClass]
public partial class Selector : BTComposite //Runs each child node in order until one succeeds. Returns Success if one succeeds, returns Failure if all children fail.
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		foreach (var child in GetChildren())
		{
			BTNode btNode = GetAsBTNode(child);
			
			BTResult result = btNode.Tick(entity, bb);
			if (result != BTResult.Failure)
			{
				return result;
			}
		}
		return BTResult.Failure;
	}
}
