using Godot;
using System;

public partial class Sequencer : BTComposite
{
    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        foreach (var child in GetChildren())
        {
			BTNode btNode = child as BTNode;
			if (btNode == null)
			{
				GD.PrintErr($"Child {child.Name} is not a BTNode!");
				return BTResult.Failure;
			}

            BTResult result = btNode.Tick(entity, bb);
			if (result != BTResult.Success)
			{
				return result;
			}
        }
		return BTResult.Success;
    }
}