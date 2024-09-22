using Godot;
using System;

[GlobalClass]
public partial class IsEnteredDoorNull : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		var enteredDoor = bb.Get<Door>(BTVariable.EnteredDoor);
		if(enteredDoor == null)
		{
			GD.Print("EnteredDoor is null! (i.e. no door was enterable.)");
			return BTResult.Success;
		}
		GD.Print("EnteredDoor is NOT null!");
		return BTResult.Failure;
	}
}
