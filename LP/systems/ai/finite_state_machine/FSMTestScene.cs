using Godot;
using System;

//temporary file to test FSM Setup

public partial class FSMTestScene : Node2D
{
	private Blackboard blackboard;
	private FSMManager fsm1;
	private FSMManager fsm2;
	private Node2D entity1;
	private Node2D entity2;

	public override void _Ready()
	{
		blackboard = new Blackboard();
		entity1 = GetNode<Node2D>("Entity1");
		entity2 = GetNode<Node2D>("Entity2");

		fsm1 = new FSMManager();
		fsm2 = new FSMManager();

		AddChild(fsm1);
		AddChild(fsm2);

		fsm1.Initialize(blackboard, entity1);
		fsm2.Initialize(blackboard, entity2);

		StateFactory.CreateStates(fsm1, entity1);
		StateFactory.CreateStates(fsm2, entity2);

		fsm1.SetState("Idle");
		fsm2.SetState("Idle");

		// Set initial blackboard values
		blackboard.Set("hunger", 60.0f);
		blackboard.Set("targetPosition", new Vector2(300, 200));
		blackboard.Set("threatPosition", entity2.Position);
		blackboard.Set("safePosition", new Vector2(50, 50));
	}

	public override void _Process(double delta)
	{
		fsm1._Process(delta);
		fsm2._Process(delta);
	}
}
