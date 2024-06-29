using Godot;
using System;
using System.Collections.Generic;

// A blackboard is a shared space for data between all nodes in a behaviour tree
public class Blackboard 
{
	private Dictionary<BTVariable, object> variables = new Dictionary<BTVariable, object>();
	private readonly object lockObj = new object();

	public void Set(BTVariable variable, object value)
	{
		lock (lockObj)
		{
			variables[variable] = value;
		}
	}

	public T Get<T>(BTVariable variable)
	{
		lock (lockObj)
		{
			if (variables.ContainsKey(variable))
			{
				return (T)variables[variable];
			}
			else
			{
				return default;
			}
		}
	}
}

public enum BTVariable
{
	MousePosition, 
	DoorList,
	SelectedDoor,
	EnteredDoor,
}
