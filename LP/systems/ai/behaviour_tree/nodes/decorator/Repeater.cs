using Godot;
using System;

[GlobalClass]
public partial class Repeater : BTDecorator, BTNode //Will return running until the repeat count is reached, returns Success after it is reached (and resets currentIteration). Cannot return Failure. 
{
	[Export] int repeatCount = -1; // -1 means infinite. The repeater will continue to process it's child forever. 

	private int currentIteration = 0;

	public override void _Ready()
	{
		if(repeatCount == 0)
		{
			GD.PushWarning($"Repeater {Name} has a repeat count of 0. This will cause the repeater to never process it's child node.");
		}
	}

    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        BTNode btNode = GetAsBTNode(GetChild(0));

		btNode.Tick(entity, bb); //We don't care about the result of the child node, we just want to process it.
		currentIteration++;

		if(repeatCount != -1 && currentIteration >= repeatCount)
		{
			currentIteration = 0;
			return BTResult.Success;
		}
		return BTResult.Running;
    }
}