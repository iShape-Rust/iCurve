use alloc::vec;
use alloc::vec::Vec;

pub(crate) struct IndexBitSet {
    chunks: Vec<u64>,
}

impl IndexBitSet {

    #[inline]
    pub(crate) fn with_size(count: usize) -> Self {
        let len = count >> 6;
        Self {
            chunks: vec![0; len],
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        for chunk in self.chunks.iter_mut() {
            *chunk = 0;
        }
    }

    #[inline]
    pub(crate) fn insert(&mut self, index: usize) {
        let chunk_index = index >> 6;
        if chunk_index >= self.chunks.len() {
            self.chunks.resize(chunk_index + 1, 0);
        }
        let bit_index = 63 & index;
        self.chunks[chunk_index] |= 1 << bit_index;
    }

    #[inline]
    pub(crate) fn contains(&self, index: usize) -> bool {
        let chunk_index = index >> 6;
        let bit_index = 63 & index;
        let chunk= self.chunks[chunk_index];
        chunk & (1 << bit_index) != 0
    }

    #[inline]
    pub(crate) fn iter(&self) -> IndexBitSetIterator {
        IndexBitSetIterator::with_buffer(self)
    }
}

pub(crate) struct IndexBitSetIterator<'a> {
    buffer: &'a [u64],
    index: usize,
    chunk: u64,
}

impl<'a> IndexBitSetIterator<'a> {
    #[inline]
    pub fn with_buffer(bitset: &'a IndexBitSet) -> Self {
        let chunk = if let Some(&first) = bitset.chunks.first() {
            first
        } else {
            0
        };
        Self { buffer: &bitset.chunks, index: 0, chunk }
    }
}

impl<'a> Iterator for IndexBitSetIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk == 0 {
            let pos = if let Some(next) = self.buffer.iter().skip(self.index + 1).position(|&ch| ch != 0) {
                next
            } else {
                return None
            };
            self.index = pos + self.index + 1;
            self.chunk = self.buffer[self.index];
        };

        let bit = self.chunk.trailing_zeros() as usize;
        let item = (self.index << 6) + bit;
        self.chunk &= !(1 << bit);

        Some(item)
    }
}

impl Default for IndexBitSet {
    fn default() -> Self {
        Self {
            chunks: vec![0; 8],
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::vec;
    use super::*;
    use std::collections::HashSet;
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    #[test]
    fn test_empty() {
        let set = IndexBitSet::default();
        let mut result = Vec::new();
        for i in set.iter() {
            result.push(i);
        }
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_value() {
        let mut set = IndexBitSet::default();
        set.insert(42);
        let mut result = Vec::new();
        for i in set.iter() {
            result.push(i);
        }
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_duplicates() {
        let mut set = IndexBitSet::default();
        set.insert(10);
        set.insert(10);
        set.insert(10);
        let mut result = Vec::new();
        for i in set.iter() {
            result.push(i);
        }
        assert_eq!(result, vec![10]);
    }

    #[test]
    fn test_ordered_values() {
        let mut set = IndexBitSet::default();
        for i in 0..100 {
            set.insert(i);
        }
        let mut result = Vec::new();
        for i in set.iter() {
            result.push(i);
        }
        let expected: Vec<_> = (0..100).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_random_comparison_with_hashset() {
        let mut rng = StdRng::seed_from_u64(12345);
        let mut set = IndexBitSet::default();
        let mut hash = HashSet::new();

        for _ in 0..10_000 {
            let v = rng.random_range(0..10_000);
            set.insert(v);
            hash.insert(v);
        }

        let mut result = Vec::new();
        for i in set.iter() {
            result.push(i);
        }
        let set_from_index_set: HashSet<_> = result.into_iter().collect();

        assert_eq!(set_from_index_set, hash);
    }

    #[test]
    fn test_reuse_and_clean() {
        let mut set = IndexBitSet::default();
        let mut result = Vec::new();

        set.insert(1);
        set.insert(2);
        for i in set.iter() {
            result.push(i);
        }
        assert_eq!(result.len(), 2);


        // Second call should be empty now
        set.clear();
        result.clear();
        for i in set.iter() {
            result.push(i);
        }
        assert!(result.is_empty());

        // Reuse
        set.insert(5);
        set.insert(10);
        for i in set.iter() {
            result.push(i);
        }
        let out_set: HashSet<_> = result.iter().cloned().collect();
        assert_eq!(out_set, HashSet::from([5, 10]));
    }

}
