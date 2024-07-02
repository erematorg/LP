using Godot;
using System;

public static class StateFactory
{
	public static void CreateStates(FSMManager fsm, Node2D entity)
	{
		fsm.AddState("Idle", new IdleState(fsm, entity));
		fsm.AddState("Searching", new SearchingState(fsm, entity));
		fsm.AddState("Attacking", new AttackState(fsm, entity));
		fsm.AddState("Fleeing", new FleeState(fsm, entity));
		fsm.AddState("Hiding", new HideState(fsm, entity));
	}

	public static IState CreateState(string stateName, FSMManager fsm, Node2D entity)
	{
		switch (stateName)
		{
			case "Idle":
				return new IdleState(fsm, entity);
			case "Searching":
				return new SearchingState(fsm, entity);
			case "Attacking":
				return new AttackState(fsm, entity);
			case "Fleeing":
				return new FleeState(fsm, entity);
			case "Hiding":
				return new HideState(fsm, entity);
			default:
				throw new ArgumentException($"Unknown state: {stateName}");
		}
	}
}

