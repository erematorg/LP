using Godot;
using System;

[GlobalClass]
public partial class CloseDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		if(selectedDoor.smashed)
		{
			GD.Print($"{selectedDoor} is smashed, cannot close it! ¯\\_(ツ)_/¯");
			return BTResult.Failure;
		}
		GD.Print($"Closed {selectedDoor}!");
		return BTResult.Success;
	}
}
