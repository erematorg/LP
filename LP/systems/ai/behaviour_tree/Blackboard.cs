using Godot;
using System;
using System.Collections.Generic;

//A blackboard is a shared space for data between all nodes in a behaviour tree
public class Blackboard 
{
	private Dictionary<BTVariable, object> variables = new Dictionary<BTVariable, object>();

    public void Set(BTVariable variable, object value)
    {
        variables[variable] = value;
    }

    public T Get<T>(BTVariable variable)
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

public enum BTVariable
{
	MousePosition, 
    DoorList,
    SelectedDoor,
    EnteredDoor,
}