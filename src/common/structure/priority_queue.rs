use std::collections::BinaryHeap;

struct PriorityQueueItem<T, A: Ord + PartialEq + Eq + PartialOrd> {
    item: T,
    cost: A,
}

impl<T, A: Ord + PartialEq + Eq + PartialOrd> Eq for PriorityQueueItem<T, A> {}

impl<T, A: Ord + PartialEq + Eq + PartialOrd> PartialEq for PriorityQueueItem<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}
impl<T, A: Ord + PartialEq + Eq + PartialOrd> PartialOrd for PriorityQueueItem<T, A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cost.cmp(&other.cost))
    }
}
impl<T, A: Ord + PartialEq + Eq + PartialOrd> Ord for PriorityQueueItem<T, A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost.cmp(&other.cost)
    }
}

pub struct PriorityQueue<T, A: Ord + PartialEq + Eq + PartialOrd> {
    heap: BinaryHeap<PriorityQueueItem<T, A>>,
}

#[allow(dead_code)]
impl<T, A: Ord + PartialEq + Eq + PartialOrd> PriorityQueue<T, A> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn put(&mut self, item: T, cost: A) {
        self.heap.push(PriorityQueueItem { cost, item })
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(item) = self.heap.pop() {
            return Some(item.item);
        }

        None
    }

    pub fn peek(&self) -> Option<&T> {
        if let Some(item) = self.heap.peek() {
            return Some(&item.item);
        }

        None
    }
}
