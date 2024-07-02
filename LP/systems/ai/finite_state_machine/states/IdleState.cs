using Godot;
using System;

public partial class IdleState : StateBase
{
	public IdleState(FSMManager fsm, Node2D entity) : base(fsm, entity) { }

	public override void Enter()
	{
		GD.Print("Entering Idle State");
	}

	public override void Execute(Blackboard blackboard, float delta)
	{
		if (blackboard.Get<float>("hunger") > 50)
		{
			fsm.SetState("Searching");
		}
	}

	public override void Exit()
	{
		GD.Print("Exiting Idle State");
	}
}
