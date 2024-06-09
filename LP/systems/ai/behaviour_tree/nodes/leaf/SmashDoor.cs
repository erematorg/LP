using Godot;
using System;

[GlobalClass]
public partial class SmashDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		GD.Print($"Trying to smash {selectedDoor}...");

		if(selectedDoor.smashable)
		{
			selectedDoor.smashed = true;
			return BTResult.Success;
		}
		return BTResult.Failure;
	}
}
