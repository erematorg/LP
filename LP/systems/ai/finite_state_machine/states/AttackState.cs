using Godot;
using System;

public partial class AttackState : StateBase
{
	public AttackState(FSMManager fsm, Node2D entity) : base(fsm, entity) { }

	public override void Enter()
	{
		GD.Print("Entering Attack State");
	}

	public override void Execute(Blackboard blackboard, float delta)
	{
		GD.Print("Attacking the target!");
		// Implement attack logic
		fsm.SetState("Fleeing");
	}

	public override void Exit()
	{
		GD.Print("Exiting Attack State");
	}
}
