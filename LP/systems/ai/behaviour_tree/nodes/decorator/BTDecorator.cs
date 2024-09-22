using Godot;
using System;

[GlobalClass]
public partial class BTDecorator : Node, BTNode
{
	// Flags for different decorator behaviors
	[Export] public bool InvertResult = false;
	[Export] public bool RepeatUntilFail = false;
	[Export] public int RepeatCount = -1; // -1 means infinite repeat

	private int currentIteration = 0;

	// Mark Tick as virtual so child classes can override it
	public virtual BTResult Tick(Entity entity, Blackboard bb)
	{
		BTNode childNode = GetAsBTNode(GetChild(0));
		BTResult result = childNode.Tick(entity, bb);

		// Invert the result if required
		if (InvertResult)
		{
			result = Invert(result);
		}

		// Handle repeat logic
		if (RepeatUntilFail && result != BTResult.Failure)
		{
			return BTResult.Running;
		}

		if (RepeatCount != -1 && currentIteration >= RepeatCount)
		{
			return BTResult.Success;
		}

		currentIteration++;
		return result;
	}

	private BTResult Invert(BTResult result)
	{
		switch (result)
		{
			case BTResult.Success:
				return BTResult.Failure;
			case BTResult.Failure:
				return BTResult.Success;
			default:
				return result;
		}
	}

	public BTNode GetAsBTNode(Node node)
	{
		if (node is BTNode btNode)
			return btNode;
		else
			throw new Exception($"{node.Name} is not a BTNode!");
	}
}
