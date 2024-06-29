using Godot;
using System;
using System.Collections.Generic;

[GlobalClass]
public partial class GetDoorList : BTAction
{

	[Export] int maxDoors = 10;
	[Export] float propertyChance = 0.6f;

	private int doorCount;

	//Only for example demonstration
	//Will create upto {maxDoors} doors
	//{propertyChance} chance for one random door to be either openable, unlockable, or smashable.
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		List<Door> doorList = new List<Door>();
		bb.Set(BTVariable.DoorList, doorList);

		doorCount = GD.RandRange(1, maxDoors);
		for (int i = 0; i < doorCount; i++)
		{
			Door door = new Door();
			door.index = i;
			doorList.Add(door);
		}

		(int doorIndex, string property)? result = SetRandomDoorProperty(doorList);
		if(result == null)
		{
			GD.PrintRich($"[font_size=17]Created {doorCount} doors and [color=red]none of them can be entered.[/color][/font_size]");
			return BTResult.Success;
		}
		else
		{
			GD.PrintRich($"[font_size=17]Created {doorCount} doors and door {result.Value.doorIndex} is {result.Value.property}[/font_size]");
			return BTResult.Success;
		}
		
	}

    private (int, string)? SetRandomDoorProperty(List<Door> doorList)
    {
		bool shouldSetProperty = GD.Randf() < propertyChance; //chance that a door will have a property
		if(!shouldSetProperty) return null;

        int randomIndex = GD.RandRange(0, doorCount - 1);
        Door randomDoor = doorList[randomIndex];
		int randomProperty = GD.RandRange(0, 2);

		switch(randomProperty)
		{	
			case 0:
				randomDoor.openable = true;
				return (randomIndex, "[wave amp=50.0 freq=5.0 connected=1]openable[/wave]");
			case 1:
				randomDoor.unlockable = true;
				return (randomIndex, "[pulse freq=1.0 ease=-2.0]unlockable[/pulse]");
			case 2:
				randomDoor.smashable = true;
				return (randomIndex, "[shake rate=20.0 level=5 connected=1]smashable[/shake]");
		}
		throw new Exception("CodeShouldNotReachHereException");
    }
}
