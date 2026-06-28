#[derive(Debug, Clone, Copy)]
pub struct StackVec<T: Copy + Default, const CAP: usize> {
    pub(crate) buffer: [T; CAP],
    pub(crate) len: usize,
}

impl<T: Copy + Default, const CAP: usize> StackVec<T, CAP> {
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); CAP],
            len: 0,
        }
    }

    #[inline]
    pub(crate) fn from_slice(src: &[T]) -> Self {
        assert!(src.len() <= CAP);

        let mut buffer = [T::default(); CAP];
        buffer[0..src.len()].copy_from_slice(src);

        Self {
            buffer,
            len: src.len(),
        }
    }

    #[inline]
    pub(crate) fn init_with(&mut self, src: &[T]) {
        assert!(src.len() <= CAP);

        self.buffer[0..src.len()].copy_from_slice(src);
        self.len = src.len();
    }

    #[inline]
    pub(crate) fn extend_from_slice(&mut self, src: &[T]) {
        assert!(self.len + src.len() <= CAP);

        self.buffer[self.len..self.len + src.len()].copy_from_slice(src);
        self.len += src.len();
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.buffer[0..self.len]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.buffer[0..self.len]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == CAP
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < CAP);

        self.buffer[self.len] = value;
        self.len += 1;
    }

    #[inline]
    pub fn push_some(&mut self, value: Option<T>) {
        if let Some(v) = value {
            self.push(v);
        }
    }

    #[inline]
    pub(crate) fn dedup(&mut self)
    where
        T: PartialEq,
    {
        if self.len <= 1 {
            return;
        }

        let mut write_index = 1;

        for read_index in 1..self.len {
            let value = self.buffer[read_index];
            if value == self.buffer[write_index - 1] {
                continue;
            }

            self.buffer[write_index] = value;
            write_index += 1;
        }

        self.len = write_index;
    }

    #[inline]
    pub fn swap_remove(&mut self, idx: usize) {
        assert!(idx < self.len);

        let last = self.len - 1;
        self.len -= 1;
        self.buffer[idx] = self.buffer[last];
    }

    #[inline]
    pub fn swap_extract(&mut self, idx: usize) -> T {
        assert!(idx < self.len);

        let value = self.buffer[idx];
        self.swap_remove(idx);
        value
    }
}

impl<T: Copy + Default, const CAP: usize> Default for StackVec<T, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_values_inline() {
        let mut values = StackVec::<u32, 4>::new();

        values.push(10);
        values.push(20);
        values.push(30);

        assert_eq!(values.as_slice(), &[10, 20, 30]);
        assert_eq!(values.len(), 3);
        assert!(!values.is_empty());
        assert!(!values.is_full());
    }

    #[test]
    fn copies_from_slice() {
        let values = StackVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);

        assert_eq!(values.as_slice(), &[1, 2, 3, 4]);
        assert!(values.is_full());
    }

    #[test]
    fn extends_from_slice() {
        let mut values = StackVec::<u32, 4>::from_slice(&[1, 2]);

        values.extend_from_slice(&[3, 4]);

        assert_eq!(values.as_slice(), &[1, 2, 3, 4]);
        assert!(values.is_full());
    }

    #[test]
    fn extracts_by_swapping_with_last() {
        let mut values = StackVec::<u32, 4>::from_slice(&[1, 2, 3, 4]);

        assert_eq!(values.swap_extract(1), 2);

        assert_eq!(values.as_slice(), &[1, 4, 3]);
    }

    #[test]
    fn dedups_adjacent_equal_values() {
        let mut values = StackVec::<u32, 8>::from_slice(&[1, 1, 2, 2, 2, 3, 1]);

        values.dedup();

        assert_eq!(values.as_slice(), &[1, 2, 3, 1]);
    }
}
