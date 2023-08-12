use node::Node;
use task::Task;

pub mod node;
pub mod task;

pub struct Balancer<N: Node> {
    nodes: Vec<N>,
}

impl<N: Node> Default for Balancer<N> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

impl<N: Node> Balancer<N> {
    pub fn new(nodes: Vec<N>) -> Self { Self { nodes } }

    pub fn enqueue(&mut self, task: <N as Node>::Task) -> Result<(), <N as Node>::Task> {
        if let Some(node) = self
            .nodes
            .iter_mut()
            .filter(|node| task.can_run(node.id()))
            .min_by_key(|node| node.sorting())
        {
            // if a node is available (there are nodes present and it can run on one of them)
            // take the one with the smallest queue and greatest priority
            node.send_task(task);
            Ok(())
        }else{
            Err(task)
        }
    }
}
