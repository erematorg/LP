using Godot;
using System;

[GlobalClass]
public partial class BTComposite : Node, BTNode
{
	// Composite type: IsSelector determines if the composite is a Selector or Sequencer
	[Export] public bool IsSelector = true;

	public override void _Ready()
	{
		if (GetChildCount() == 0)
		{
			GD.PrintErr($"Composite {Name} must have at least one child node!");
		}
	}

	// Mark Tick as virtual so it can be overridden by child classes
	public virtual BTResult Tick(Entity entity, Blackboard bb)
	{
		foreach (Node child in GetChildren())
		{
			BTNode btNode = GetAsBTNode(child);
			BTResult result = btNode.Tick(entity, bb);

			if (IsSelector)
			{
				if (result != BTResult.Failure)
				{
					return result;
				}
			}
			else // Sequencer logic
			{
				if (result != BTResult.Success)
				{
					return result;
				}
			}
		}

		return IsSelector ? BTResult.Failure : BTResult.Success;
	}

	public BTNode GetAsBTNode(Node node)
	{
		if (node is BTNode btNode)
			return btNode;
		else
			throw new Exception($"{node.Name} is not a BTNode!");
	}
}
