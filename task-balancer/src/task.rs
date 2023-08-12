pub trait Task {
	type NodeId;
    fn can_run(&self, node: Self::NodeId) -> bool;
}
