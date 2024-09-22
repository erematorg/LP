using Godot;
using System;

public class Door
{
	public int index;

	public bool openable;
	public bool unlockable;
	public bool smashable;

	public bool smashed; 

	public override string ToString()
	{
		return "Door" + index;
	}
}
