use alloc::vec::Vec;

#[allow(dead_code)]
pub(crate) trait Merge {
    fn merge(&mut self, other: &mut Self) -> bool;
}

struct Node<T> {
    next: u32,
    value: T,
}
pub(crate) struct CircularMergeList<T> {
    nodes: Vec<Node<T>>,
    head: u32,
    len: u32,
}

impl<T> CircularMergeList<T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            head: 0,
            len: 0,
        }
    }

    pub(crate) fn merge_with<M>(&mut self, values: Vec<T>, mut merge: M) -> Vec<T>
    where
        M: FnMut(&mut T, &mut T) -> bool,
    {
        self.init(values);
        self.merge_nodes(&mut merge);
        self.extract_vec()
    }

    fn init(&mut self, values: Vec<T>) {
        self.nodes.clear();
        self.head = 0;

        if values.is_empty() {
            self.len = 0;
            return;
        }

        assert!(values.len() <= u32::MAX as usize);

        self.len = values.len() as u32;
        self.nodes.reserve(values.len());
        for (next, value) in (1..).zip(values) {
            self.nodes.push(Node { next, value });
        }

        let last_index = self.nodes.len() - 1;
        self.nodes[last_index].next = 0;
    }

    fn merge_nodes<M>(&mut self, merge: &mut M)
    where
        M: FnMut(&mut T, &mut T) -> bool,
    {
        if self.len < 2 {
            return;
        }

        let mut index = self.head as usize;
        let mut checked_count = 0u32;

        while self.len > 1 && checked_count < self.len {
            let next = self.nodes[index].next as usize;
            let next_next = self.nodes[next].next;

            if self.merge_pair(index, next, merge) {
                self.nodes[index].next = next_next;
                self.len -= 1;
                checked_count = 0;

                if next as u32 == self.head {
                    self.head = index as u32;
                }
            } else {
                index = next;
                checked_count += 1;
            }
        }
    }

    fn extract_vec(&mut self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len as usize);

        if self.len == 0 {
            return result;
        }

        let mut index = self.head as usize;
        let mut nodes: Vec<Option<Node<T>>> = self.nodes.drain(..).map(Some).collect();

        for _ in 0..self.len {
            let node = nodes[index].take().unwrap();

            index = node.next as usize;
            result.push(node.value);
        }

        self.head = 0;
        self.len = 0;

        result
    }

    fn merge_pair<M>(&mut self, index: usize, next: usize, merge: &mut M) -> bool
    where
        M: FnMut(&mut T, &mut T) -> bool,
    {
        if index < next {
            let (left, right) = self.nodes.split_at_mut(next);
            merge(&mut left[index].value, &mut right[0].value)
        } else {
            let (left, right) = self.nodes.split_at_mut(index);
            merge(&mut right[0].value, &mut left[next].value)
        }
    }
}

impl<T: Merge> CircularMergeList<T> {
    #[allow(dead_code)]
    pub(crate) fn merge(&mut self, values: Vec<T>) -> Vec<T> {
        self.merge_with(values, |a, b| a.merge(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    impl Merge for u32 {
        fn merge(&mut self, other: &mut Self) -> bool {
            self == other
        }
    }

    impl Merge for Vec<u32> {
        fn merge(&mut self, other: &mut Self) -> bool {
            let mut result = Vec::new();

            for value in self.iter() {
                if other.contains(value) && !result.contains(value) {
                    result.push(*value);
                }
            }

            if result.is_empty() {
                false
            } else {
                *self = result;
                true
            }
        }
    }

    #[test]
    fn test_0() {
        let mut list = CircularMergeList::with_capacity(4);

        assert_eq!(list.merge(vec![2, 2, 3, 2]), vec![2, 3]);
    }

    #[test]
    fn test_1() {
        let mut list = CircularMergeList::with_capacity(4);

        assert_eq!(list.merge(vec![2, 2, 3, 3]), vec![2, 3]);
    }

    #[test]
    fn test_2() {
        let mut list = CircularMergeList::with_capacity(4);

        assert_eq!(list.merge(vec![2, 3, 3, 2]), vec![2, 3]);
    }

    #[test]
    fn test_3() {
        let mut list = CircularMergeList::with_capacity(4);

        assert_eq!(list.merge(vec![3, 3, 3, 3]), vec![3]);
    }

    #[test]
    fn test_4() {
        let mut list = CircularMergeList::with_capacity(3);

        assert_eq!(
            list.merge(vec![vec![1, 2], vec![2, 3], vec![1, 3]]),
            vec![vec![2], vec![1, 3]]
        );
    }

    #[test]
    fn test_5() {
        let mut list = CircularMergeList::with_capacity(3);

        assert_eq!(
            list.merge(vec![vec![1, 2], vec![1, 2], vec![1, 3]]),
            vec![vec![1]]
        );
    }

    #[test]
    fn test_6() {
        let mut list = CircularMergeList::with_capacity(3);

        assert_eq!(
            list.merge(vec![vec![1, 2, 3], vec![1, 2], vec![2, 3]]),
            vec![vec![2]]
        );
    }
}
