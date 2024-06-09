using Godot;
using System;

[GlobalClass]
public partial class WalkToDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		GD.Print($"Walking to {selectedDoor}...");
		return BTResult.Success;
	}
}
