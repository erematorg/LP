using Godot;
using System;

[GlobalClass]
public partial class OpenDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		GD.Print($"Trying to open {selectedDoor}...");

		if(selectedDoor.openable)
		{
			return BTResult.Success;
		}
		return BTResult.Failure;
	}
}
