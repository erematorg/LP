using Godot;
using System;

[GlobalClass]
public abstract partial class BTDecorator : Node, BTNode
{
    public abstract BTResult Tick(Entity entity, Blackboard bb);

    public override void _Ready()
    {
        if(GetChildCount() == 1)
		{
			GD.PrintErr($"Decorator {Name} must have only one child node!");
		}
    }

	//duplicate method from BTComposite. Might need to have a common parent class. Limited duplication so I'm leaving it here for now. 
	public BTNode GetAsBTNode(Node node)
    {
        if(node is BTNode btNode)
            return btNode;
        else
        {
            throw new Exception($"{node.Name} is not a BTNode!");
        }
    }
}