using Godot;
using System;

public partial class Selector : BTComposite
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
