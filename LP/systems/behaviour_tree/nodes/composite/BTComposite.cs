using Godot;
using System;

[GlobalClass]
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

    public BTNode GetAsBTNode(Node node)
    {
        if(node is BTNode btNode)
            return btNode;
        else
        {
            throw new Exception($"Child {node.Name} is not a BTNode!");
        }
    }
}
