use crate::task::Task;

#[derive(PartialEq, Eq)]
pub struct SortingPriority {
    queue_length: usize,
    priority: usize,
}

impl PartialOrd for SortingPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.queue_length.partial_cmp(&other.queue_length) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.priority
            .partial_cmp(&other.priority)
            .map(std::cmp::Ordering::reverse)
    }
}

impl Ord for SortingPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.queue_length
            .cmp(&other.queue_length)
            .then_with(|| self.priority.cmp(&other.priority).reverse())
    }
}

pub trait Node {
    type Id: Copy;
    type Task: Task<NodeId = Self::Id>;

    fn send_task(&mut self, task: Self::Task);
    fn queue_length(&self) -> usize;
    fn priority(&self) -> usize;
    fn id(&self) -> Self::Id;
    fn sorting(&self) -> SortingPriority {
        SortingPriority {
            queue_length: self.queue_length(),
            priority: self.priority(),
        }
    }
}

impl<N> Node for &'_ mut N
where
    N: Node,
{
    type Id = N::Id;

    type Task = N::Task;

    fn send_task(&mut self, task: Self::Task) {
        (**self).send_task(task)
    }

    fn queue_length(&self) -> usize {
        (**self).queue_length()
    }

    fn priority(&self) -> usize {
        (**self).priority()
    }

    fn id(&self) -> Self::Id {
        (**self).id()
    }
}
