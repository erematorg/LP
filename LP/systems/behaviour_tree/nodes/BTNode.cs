using Godot;
using System;

public interface BTNode
{
	public BTResult Tick(Entity entity, Blackboard bb);
}
