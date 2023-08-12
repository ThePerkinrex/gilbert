use task_balancer::{task::Task, node::Node, Balancer};

struct MockTask {
	can_run: Vec<usize>,
	id: usize
}

impl Task for MockTask {
    type NodeId = usize;

    fn can_run(&self, node: Self::NodeId) -> bool {
        self.can_run.contains(&node)
    }
}

struct MockNode {
	queue: Vec<usize>,
	priority: usize,
	id: usize
}

impl MockNode {
    fn new(priority: usize, id: usize) -> Self { Self { queue: Default::default(), priority, id } }
}

impl Node for MockNode {
    type Id = usize;
    type Task = MockTask;

    fn send_task(&mut self, task: Self::Task) {
        self.queue.push(task.id)
    }

    fn queue_length(&self) -> usize {
        self.queue.len()
    }

    fn priority(&self) -> usize {
        self.priority
    }

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[test]
fn single_node() {
	let mut node = MockNode::new(0, 0);
	let mut balancer = Balancer::new(vec![&mut node]);
	assert!(balancer.enqueue(MockTask {can_run: vec![0], id: 0}).is_ok());
	assert!(balancer.enqueue(MockTask {can_run: vec![0,1,2], id: 1}).is_ok());
	assert!(balancer.enqueue(MockTask {can_run: vec![1,2], id: 2}).is_err());
	assert_eq!(node.queue, vec![0,1]);
}

#[test]
fn multi_node() {
	let mut node_a = MockNode::new(0, 0);
	let mut node_b = MockNode::new(0, 1);
	let mut balancer = Balancer::new(vec![&mut node_a, &mut node_b]);
	assert!(balancer.enqueue(MockTask {can_run: vec![0], id: 0}).is_ok());
	assert!(balancer.enqueue(MockTask {can_run: vec![0,1,2], id: 1}).is_ok());
	assert!(balancer.enqueue(MockTask {can_run: vec![1,2], id: 2}).is_ok());
	assert_eq!(node_a.queue, vec![0]);
	assert_eq!(node_b.queue, vec![1,2]);
}
