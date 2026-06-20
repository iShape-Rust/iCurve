use alloc::vec::Vec;
use crate::bool::overlay::CurveOverlay;
use crate::bool::slice::collider::Collider;
use i_key_sort::sort::key::SortKey;
use i_key_sort::sort::two_keys::TwoKeysSort;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::collections::stack_vec::StackVec;

pub(crate) struct SliceBuffer<T: FloatNumber, I: IntNumber> {
    colliders: Vec<Collider<T, I>>,
    indices: Vec<usize>,
}

impl<T: FloatNumber, I: IntNumber> Default for SliceBuffer<T, I> {
    fn default() -> Self {
        Self { colliders: Vec::new(), indices: Vec::new()}
    }
}

impl<P: FloatPointCompatible, I: IntNumber + SortKey> CurveOverlay<P, I> {
    pub(crate) fn slice(&mut self) {
        self.create_and_sort_original_colliders();
        self.filter_original_colliders();

    }

    fn create_and_sort_original_colliders(&mut self) {
        self.slice_buffer.colliders.clear();
        self.slice_buffer.colliders.reserve(self.segments.len());
        for (i, s) in self.segments.iter().enumerate() {
            self.slice_buffer.colliders.push(Collider::new(i, s.segment, &self.internal_adapter));
        }
        self.slice_buffer.colliders.sort_by_two_keys(false, |col| col.rect.min_x, |col| col.rect.min_y);
    }

    fn filter_original_colliders(&mut self) {
        let n = self.slice_buffer.colliders.len();
        let mut a_convex: StackVec<IntPoint, 4> = StackVec::new();
        let mut b_convex: StackVec<IntPoint, 4> = StackVec::new();

        for (i, a) in self.slice_buffer.colliders.iter().take(n - 1).enumerate() {
            a_convex.clear();
            let mut j = i + 1;
            while j < n {
                let b = &self.slice_buffer.colliders[j];
                if a.index == b.index || !a.rect.is_intersect_border_include(&b.rect) {
                    j += 1;
                    continue;
                }

                j += 1;
            }
        }
    }
}
