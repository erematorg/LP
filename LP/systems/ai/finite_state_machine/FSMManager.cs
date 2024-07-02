using Godot;
using System;
using System.Collections.Generic;

/// FSMManager handles the state transitions and execution of states.
public partial class FSMManager : Node
{
	private IState currentState;
	private Dictionary<string, IState> states = new Dictionary<string, IState>();
	public Blackboard blackboard;
	public Node2D entity;

	public FSMManager() {}

	public void Initialize(Blackboard sharedBlackboard, Node2D entity)
	{
		blackboard = sharedBlackboard ?? throw new ArgumentNullException(nameof(sharedBlackboard));
		this.entity = entity ?? throw new ArgumentNullException(nameof(entity));
	}

	/// Adds a state to the FSM.
	/// <param name="name">The name of the state.</param>
	/// <param name="state">The state instance.</param>
	public void AddState(string name, IState state)
	{
		if (state == null) throw new ArgumentNullException(nameof(state));

		if (!states.ContainsKey(name))
		{
			states[name] = state;
		}
		else
		{
			GD.PrintErr($"State {name} already exists. Overwriting is not allowed.");
		}
	}

	/// Removes a state from the FSM.
	/// <param name="name">The name of the state to remove.</param>
	public void RemoveState(string name)
	{
		if (states.ContainsKey(name))
		{
			states.Remove(name);
		}
		else
		{
			GD.PrintErr($"State {name} does not exist and cannot be removed.");
		}
	}

	/// Sets the current state of the FSM.
	/// <param name="name">The name of the state to switch to.</param>
	public void SetState(string name)
	{
		if (states.ContainsKey(name))
		{
			SwitchState(states[name]);
		}
		else
		{
			IState newState = StateFactory.CreateState(name, this, entity);
			if (newState != null)
			{
				AddState(name, newState);
				SwitchState(newState);
			}
			else
			{
				GD.PrintErr($"Failed to create state: {name}");
			}
		}
	}

	/// Switches the current state to a new state.
	/// <param name="newState">The new state to switch to.</param>
	private void SwitchState(IState newState)
	{
		currentState?.Exit();
		currentState = newState;
		currentState.Enter();
	}

	/// Processes the current state each frame.
	/// <param name="delta">The time elapsed since the last frame.</param>
	public override void _Process(double delta)
	{
		currentState?.Execute(blackboard, (float)delta);
	}
}
