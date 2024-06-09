using Godot;
using System;

[GlobalClass]
public partial class BehaviourTree : Node
{
	private Blackboard blackboard = new Blackboard();
	private BTNode childBTNode;
	private Entity ownerEntity;

	private bool success;

	public override void _Ready()
	{
		ownerEntity = GetOwner<Entity>();

		if(GetChildCount() != 1)
		{
			throw new Exception("BehaviourTree must have exactly one child BTNode!");
		}
		if(GetChild(0) is not BTNode btNode)
		{
			throw new Exception("BehaviourTree's child must be a BTNode!");
		}
		childBTNode = btNode;
	}

	public override void _Process(double delta)
	{
		if(!success)
		{
			var result = childBTNode.Tick(ownerEntity, blackboard);
			if(result == BTResult.Success)
			{
				success = true;
			}
		}	
	}
}
