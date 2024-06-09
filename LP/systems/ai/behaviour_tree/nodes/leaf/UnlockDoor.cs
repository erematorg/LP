using Godot;
using System;

[GlobalClass]
public partial class UnlockDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		GD.Print($"Trying to unlock {selectedDoor}...");

		if(selectedDoor.unlockable)
		{
			return BTResult.Success;
		}
		return BTResult.Failure;
	}
}
