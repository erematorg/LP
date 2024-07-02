public interface IState
{
	void Enter();
	void Execute(Blackboard blackboard, float delta);
	void Exit();
}
