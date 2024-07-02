using Godot;
using System;

public partial class FleeState : StateBase
{
	private float messageCooldown = 0.0f; // Add cooldown for messages

	public FleeState(FSMManager fsm, Node2D entity) : base(fsm, entity) { }

	public override void Enter()
	{
		GD.Print("Entering Flee State");
	}

	public override void Execute(Blackboard blackboard, float delta)
	{
		messageCooldown -= delta; // Decrease cooldown by delta time

		if (messageCooldown <= 0)
		{
			GD.Print("Fleeing from threat...");
			messageCooldown = 1.0f; // Reset cooldown to 1 second
		}

		Vector2 threatPosition = blackboard.Get<Vector2>("threatPosition");
		Vector2 safePosition = blackboard.Get<Vector2>("safePosition");

		Vector2 fleeDirection = (entity.Position - threatPosition).Normalized();
		entity.Position += fleeDirection * 2.0f * delta; // Adjust speed

		if (entity.Position.DistanceTo(safePosition) < 10.0f)
		{
			fsm.SetState("Hiding");
		}
	}

	public override void Exit()
	{
		GD.Print("Exiting Flee State");
	}
}
