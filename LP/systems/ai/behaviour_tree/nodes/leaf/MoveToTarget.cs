using Godot;
using System;

[GlobalClass]
public partial class MoveToTarget : BTAction
{
    [Export] BTVariable targetPosition;

    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        GD.Print("Moving...");
		entity.MoveToPosition(bb.Get<Vector2>(targetPosition));
		return BTResult.Running;
    }
}
