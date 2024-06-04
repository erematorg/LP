using Godot;
using System;

public interface BTNode
{
	public abstract BTResult Tick(Entity entity, Blackboard bb);
}
