using Godot;
using System;

public partial class SearchingState : StateBase
{
	public SearchingState(FSMManager fsm, Node2D entity) : base(fsm, entity) { }

	public override void Enter()
	{
		GD.Print("Entering Searching State");
	}

	public override void Execute(Blackboard blackboard, float delta)
	{
		Vector2 targetPosition = blackboard.Get<Vector2>("targetPosition");

		entity.Position = entity.Position.Lerp(targetPosition, 0.01f); // Slower movement

		if (entity.Position.DistanceTo(targetPosition) < 10.0f)
		{
			fsm.SetState("Attacking");
		}
	}

	public override void Exit()
	{
		GD.Print("Exiting Searching State");
	}
}
