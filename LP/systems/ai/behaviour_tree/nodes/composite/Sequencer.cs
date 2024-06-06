using Godot;
using System;

[GlobalClass]
public partial class Sequencer : BTComposite
{
    public override BTResult Tick(Entity entity, Blackboard bb)
    {
        foreach (var child in GetChildren())
        {
			BTNode btNode = GetAsBTNode(child);

            BTResult result = btNode.Tick(entity, bb);
			if (result != BTResult.Success)
			{
				return result;
			}
        }
		return BTResult.Success;
    }
}