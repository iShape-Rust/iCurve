use crate::bool::overlay::CurveOverlay;
use crate::bool::slice::collider::Collider;
use crate::collections::stack_vec::StackVec;
use crate::kernel::float::curve::segment::FloatSegment;
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_key_sort::sort::two_keys::TwoKeysSort;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(crate) struct SliceBuffer<T: FloatNumber, I: IntNumber> {
    colliders: Vec<Collider<T, I>>,
    convexes: Vec<StackVec<IntPoint<I>, 4>>,
    indices: Vec<usize>,
}

impl<T: FloatNumber, I: IntNumber> Default for SliceBuffer<T, I> {
    fn default() -> Self {
        Self {
            colliders: Vec::new(),
            convexes: Vec::new(),
            indices: Vec::new(),
        }
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
            self.slice_buffer
                .colliders
                .push(Collider::new(i, s.segment, &self.internal_adapter));
        }
        self.slice_buffer
            .colliders
            .sort_by_two_keys(false, |col| col.rect.min_x, |col| col.rect.min_y);
    }

    fn filter_original_colliders(&mut self) {
        let n = self.slice_buffer.colliders.len();
        self.slice_buffer.convexes.clear();
        self.slice_buffer.convexes.reserve(n);
        for collider in self.slice_buffer.colliders.iter() {
            self.slice_buffer
                .convexes
                .push(collider.segment.to_convex(&self.internal_adapter));
        }

        for (i, (a_collider, a_convex)) in self
            .slice_buffer
            .colliders
            .iter()
            .take(n - 1)
            .zip(self.slice_buffer.convexes.iter())
            .enumerate()
        {
            let mut j = i + 1;
            while j < n {
                let b_collider = &self.slice_buffer.colliders[j];
                if a_collider.index == b_collider.index
                    || !a_collider.rect.is_intersect_border_include(&b_collider.rect)
                {
                    j += 1;
                    continue;
                }

                let b_convex = self.slice_buffer.convexes[j];

                // a_convex.

                j += 1;
            }
        }
    }
}

trait ConvexStore<T: FloatNumber, I: IntNumber> {
    fn get(
        &mut self,
        index: usize,
        segment: &FloatSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> StackVec<IntPoint<I>, 4>;
}

impl<T: FloatNumber, I: IntNumber> ConvexStore<T, I> for Vec<StackVec<IntPoint<I>, 4>> {
    #[inline]
    fn get(
        &mut self,
        index: usize,
        segment: &FloatSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> StackVec<IntPoint<I>, 4> {
        let mut value = self[index];
        if value.is_empty() {
            value = segment.to_convex(adapter);
            self[index] = value;
        }
        value
    }
}
