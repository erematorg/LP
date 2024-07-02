using Godot;
using System;
using System.Collections.Generic;

// A blackboard is a shared space for data between all nodes in a behaviour tree
public class Blackboard 
{
	private Dictionary<object, object> variables = new Dictionary<object, object>();
	private readonly object lockObj = new object();

	// Overload Set method to accept BTVariable keys
	public void Set(BTVariable variable, object value)
	{
		lock (lockObj)
		{
			variables[variable] = value;
		}
	}

	// Overload Set method to accept string keys
	public void Set(string key, object value)
	{
		lock (lockObj)
		{
			variables[key] = value;
		}
	}

	// Overload Get method to accept BTVariable keys
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

	// Overload Get method to accept string keys
	public T Get<T>(string key)
	{
		lock (lockObj)
		{
			if (variables.ContainsKey(key))
			{
				return (T)variables[key];
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
