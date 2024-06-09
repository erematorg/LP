using Godot;
using System;

[GlobalClass]
public partial class BehaviourTree : Node
{
	private Blackboard blackboard = new Blackboard();
	private BTNode childBTNode;
	private Entity ownerEntity;

	private bool treeEnded;

	public override void _Ready()
	{
		ownerEntity = GetOwner<Entity>();

		if (GetChildCount() != 1)
		{
			throw new Exception("BehaviourTree must have exactly one child BTNode!");
		}
		if (GetChild(0) is not BTNode btNode)
		{
			throw new Exception("BehaviourTree's child must be a BTNode!");
		}
		childBTNode = btNode;
		treeEnded = false;
	}

	public override void _Process(double delta)
	{
		if (!treeEnded)
		{
			var result = childBTNode.Tick(ownerEntity, blackboard);
			string message = "[font_size=17]";

			if (result == BTResult.Success)
			{
				treeEnded = true;
				Door enteredDoor = blackboard.Get<Door>(BTVariable.EnteredDoor);
				message += $"[color=green]BehaviourTree has finished executing. {enteredDoor} was successfully entered![/color]";
			}
			else if (result == BTResult.Failure)
			{
				treeEnded = true;
				message += "[color=green]BehaviourTree has finished executing. No doors were openable, unlockable or smashable.[/color]";
			}

			if (treeEnded)
			{
				message += "[/font_size]";
				GD.PrintRich(message);
			}
		}
	}
}

