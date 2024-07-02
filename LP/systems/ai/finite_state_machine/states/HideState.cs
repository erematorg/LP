using Godot;
using System;

public partial class HideState : StateBase
{
	public HideState(FSMManager fsm, Node2D entity) : base(fsm, entity) { }

	public override void Enter()
	{
		GD.Print("Entering Hide State");
	}

	public override void Execute(Blackboard blackboard, float delta)
	{
		GD.Print("Hiding from threat...");
		// Implement hiding logic
		fsm.SetState("Idle");
	}

	public override void Exit()
	{
		GD.Print("Exiting Hide State");
	}
}
