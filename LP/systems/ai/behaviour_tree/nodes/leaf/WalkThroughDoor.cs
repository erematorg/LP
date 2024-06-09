using Godot;
using System;

[GlobalClass]
public partial class WalkThroughDoor : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		Door selectedDoor = bb.Get<Door>(BTVariable.SelectedDoor);
		GD.Print($"Walking through {selectedDoor}");
		return BTResult.Success;
	}
}
