use crate::kernel::float::cross::contact::ContactPoint;
use crate::kernel::float::cross::overlap::find::FindOverlap;
use crate::kernel::float::cross::rect::FloatRectExt;
use crate::kernel::float::cross::solver::{IntersectionResult, QuadQuadPair, Solver};
use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::curve::quad::{FloatQuadSegment, SubQuadSegment};
use crate::kernel::float::curve::split_at::FloatSplitAt;
use crate::kernel::float::math::rect::ToRect;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_quad_and_quad(
        &mut self,
        quad0: FloatQuadSegment<T>,
        quad1: FloatQuadSegment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) -> IntersectionResult<T> {
        if let Some(overlap) = quad0.find_overlap(&quad1, self.grid_size()) {
            return IntersectionResult::Overlap(overlap);
        }

        self.split_search(quad0, quad1);

        let _ = output;
        // TODO: solve each comparable quad-quad pair and push ContactPoint values.

        IntersectionResult::None
    }

    fn split_search(&mut self, quad0: FloatQuadSegment<T>, quad1: FloatQuadSegment<T>) {
        self.quad_quad_stack.clear();
        self.quad_quad_pairs.clear();

        self.quad_quad_stack.push(QuadQuadPair {
            quad0: SubQuadSegment::with_quad(quad0),
            quad1: SubQuadSegment::with_quad(quad1),
        });

        while let Some(pair) = self.quad_quad_stack.pop() {
            let rect0 = pair.quad0.quad.to_rect();
            let rect1 = pair.quad1.quad.to_rect();
            let size0 = rect0.max_size();
            let size1 = rect1.max_size();
            let epsilon = size0.max(size1) * self.relative_epsilon();

            if !rect0.is_intersect_with_padding(&rect1, epsilon) {
                continue;
            }

            let can_split0 = size0 > self.min_possible_size();
            let can_split1 = size1 > self.min_possible_size();

            if size0 >= size1 {
                if can_split0 {
                    let [left, right] = Self::split_sub_quad_at_half(pair.quad0);
                    self.quad_quad_stack.push(QuadQuadPair {
                        quad0: right,
                        quad1: pair.quad1,
                    });
                    self.quad_quad_stack.push(QuadQuadPair {
                        quad0: left,
                        quad1: pair.quad1,
                    });
                } else {
                    self.quad_quad_pairs.push(pair);
                }
            } else if can_split1 {
                let [left, right] = Self::split_sub_quad_at_half(pair.quad1);
                self.quad_quad_stack.push(QuadQuadPair {
                    quad0: pair.quad0,
                    quad1: right,
                });
                self.quad_quad_stack.push(QuadQuadPair {
                    quad0: pair.quad0,
                    quad1: left,
                });
            } else {
                self.quad_quad_pairs.push(pair);
            }
        }
    }

    #[inline]
    fn split_sub_quad_at_half(segment: SubQuadSegment<T>) -> [SubQuadSegment<T>; 2] {
        let [left, right] = segment.quad.split_at(T::HALF);
        let t0 = segment.t0.value();
        let t1 = segment.t1.value();
        let tm = (t0 + t1) * T::HALF;
        let sp = FloatSegmentParam::from(tm);

        [
            SubQuadSegment {
                quad: left,
                t0: segment.t0,
                t1: sp,
            },
            SubQuadSegment {
                quad: right,
                t0: sp,
                t1: segment.t1,
            },
        ]
    }
}

#[cfg(test)]
mod tests {}
