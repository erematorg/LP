using Godot;
using System;

public abstract partial class BTComposite : Node, BTNode
{
    public abstract BTResult Tick(Entity entity, Blackboard bb);

    public override void _Ready()
    {
        if(GetChildCount() == 0)
		{
			GD.PrintErr($"Composite {Name} must have atleast one child node!");
		}
    }
}
