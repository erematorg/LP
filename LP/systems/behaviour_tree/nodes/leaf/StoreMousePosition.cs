using Godot;
using System;

[GlobalClass]
public partial class StoreMousePosition : BTAction
{
	public override BTResult Tick(Entity entity, Blackboard bb)
	{
		bb.Set(BTVariable.MousePosition, entity.GetGlobalMousePosition());
		return BTResult.Success;
	}

}
