using Godot;
using System;

[GlobalClass]
public partial class GiveUpOnLife : BTAction
{
    public override BTResult Tick(Entity entity, Blackboard bb)
    {
		GD.Print("I give up on life...");
        return BTResult.Success;
    }
}
