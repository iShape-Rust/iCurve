use alloc::vec::Vec;

pub(crate) const EMPTY_REF: u32 = u32::MAX;

#[derive(Debug, Clone)]
pub(crate) struct ListNode<T> {
    pub(crate) prev: u32,
    pub(crate) next: u32,
    pub(crate) item: T,
}

#[derive(Debug, Clone)]
pub(crate) struct LinkList<T> {
    nodes: Vec<ListNode<T>>,
}

impl<T> LinkList<T> {
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn new(items: Vec<T>) -> Self {
        let mut nodes = Vec::with_capacity((2 * items.len()).min(16));
        if items.is_empty() {
            return Self { nodes };
        }

        let mut prev = EMPTY_REF;
        let mut next = 1;
        for item in items {
            nodes.push(ListNode { prev, next, item });
            prev = next - 1;
            next += 1;
        }

        nodes.last_mut().unwrap().next = EMPTY_REF;

        Self { nodes }
    }

    #[inline]
    pub(crate) fn get(&self, index: u32) -> &ListNode<T> {
        unsafe { self.nodes.get_unchecked(index as usize) }
    }

    pub(crate) fn split_at(&mut self, index: u32, a: T, b: T) -> (u32, u32) {
        // insert a new node as next and update this node value

        let new_index = self.nodes.len() as u32;

        let node = &mut self.nodes[index as usize];
        let next = node.next;
        node.next = new_index;
        node.item = a;

        self.nodes.push(ListNode {
            prev: index,
            next,
            item: b,
        });

        let index_next = next as usize;

        if index_next < self.nodes.len() {
            self.nodes[index_next].prev = new_index;
        }

        (index, new_index)
    }

    #[inline]
    pub(crate) fn node_iter(&self) -> NodeIterator<T> {
        NodeIterator::new(self)
    }
}

pub(crate) struct NodeIterator<'a, T> {
    list: &'a LinkList<T>,
    index: u32,
}

impl<'a, T> NodeIterator<'a, T> {
    #[inline]
    pub(crate) fn new(list: &'a LinkList<T>) -> Self {
        let index = if list.len() == 0 {
            EMPTY_REF
        } else {
            (list.len() - 1) as u32
        };
        Self { list, index }
    }
}

impl<'a, T> Iterator for NodeIterator<'a, T> {
    type Item = &'a ListNode<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == EMPTY_REF {
            return None;
        }

        let node = self.list.get(self.index);
        self.index = node.next;

        Some(&node)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use crate::data::link_list::LinkList;

    #[test]
    fn test_00() {
        let mut list = LinkList::new(vec![0, 1, 3]);
        list.split_at(1, 1, 2);

        assert_eq!(list.nodes[0].item, 0);
        assert_eq!(list.nodes[1].item, 1);
        assert_eq!(list.nodes[3].item, 2);
        assert_eq!(list.nodes[2].item, 3);
    }

    #[test]
    fn test_01() {
        let mut list = LinkList::new(vec![0, 2, 3]);
        list.split_at(0, 0, 1);

        assert_eq!(list.nodes[0].item, 0);
        assert_eq!(list.nodes[3].item, 1);
        assert_eq!(list.nodes[1].item, 2);
        assert_eq!(list.nodes[2].item, 3);
    }

    #[test]
    fn test_02() {
        let mut list = LinkList::new(vec![0, 1, 2]);
        list.split_at(2, 2, 3);

        assert_eq!(list.nodes[0].item, 0);
        assert_eq!(list.nodes[1].item, 1);
        assert_eq!(list.nodes[2].item, 2);
        assert_eq!(list.nodes[3].item, 3);
    }
}