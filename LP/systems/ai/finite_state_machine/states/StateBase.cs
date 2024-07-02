using Godot;
using System;

public abstract partial class StateBase : IState
{
	protected FSMManager fsm;
	protected Node2D entity;

	public StateBase(FSMManager fsm, Node2D entity)
	{
		this.fsm = fsm ?? throw new ArgumentNullException(nameof(fsm));
		this.entity = entity ?? throw new ArgumentNullException(nameof(entity));
	}

	public abstract void Enter();
	public abstract void Execute(Blackboard blackboard, float delta);
	public abstract void Exit();
}
