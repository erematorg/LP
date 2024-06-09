using Godot;
using System;
using System.Collections.Generic;

[GlobalClass]
public partial class PopDoorFromList : BTAction
{
	int counter = 0;

	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		List<Door> doorList = bb.Get<List<Door>>(BTVariable.DoorList);

		if(doorList.Count > 0)
		{
			bb.Set(BTVariable.SelectedDoor, doorList[0]);
			doorList.RemoveAt(0);
			GD.Print($"Popped door from list: index[{counter}]");
			counter++;
			return BTResult.Success;
		}
		else
			GD.Print("All doors have been checked!");
			return BTResult.Failure;
	}
}
