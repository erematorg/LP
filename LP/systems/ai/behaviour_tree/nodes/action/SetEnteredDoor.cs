using Godot;
using System;

[GlobalClass]
public partial class SetEnteredDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		bb.Set(BTVariable.EnteredDoor, selectedDoor); // Set the used door to the selected door
		GD.Print($"Setting {selectedDoor} as EnteredDoor...");
		return BTResult.Success;
	}
}
