use crate::kernel::cross::rect::FloatRectExt;
use crate::kernel::cross::solver::{QuadQuadPair, Solver};
use crate::kernel::cross::contact::ContactPoint;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::quad::{QuadSegment, SubQuadSegment};
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_quad_and_quad(
        &mut self,
        quad0: QuadSegment<T>,
        quad1: QuadSegment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) {
        self.collect_comparable_quad_quad_pairs(quad0, quad1);

        let _ = output;
        // TODO: solve each comparable quad-quad pair and push ContactPoint values.
    }

    fn collect_comparable_quad_quad_pairs(&mut self, quad0: QuadSegment<T>, quad1: QuadSegment<T>) {
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

            if rect0.is_comparable_size(&rect1, epsilon) {
                self.quad_quad_pairs.push(pair);
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
        let sp = SegmentParam::from(tm);

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
mod tests {
    use crate::kernel::cross::rect::FloatRectExt;
    use crate::kernel::cross::solver::Solver;
    use crate::kernel::curve::quad::QuadSegment;

    #[test]
    fn collects_comparable_pairs() {
        let quad0 = QuadSegment {
            control_points: [[0.0, 0.0].into(), [50.0, 1.0].into(), [100.0, 0.0].into()],
        };
        let quad1 = QuadSegment {
            control_points: [[45.0, -10.0].into(), [50.0, 10.0].into(), [55.0, -10.0].into()],
        };

        let mut solver: Solver<f64> = Solver::with_grid_size_and_options(0.001, 1.0e-10, 0.001);
        solver.collect_comparable_quad_quad_pairs(quad0, quad1);

        assert!(!solver.quad_quad_pairs.is_empty());

        for pair in &solver.quad_quad_pairs {
            let rect0 = pair.quad0.quad.to_rect();
            let rect1 = pair.quad1.quad.to_rect();
            let size0 = rect0.max_size();
            let size1 = rect1.max_size();
            let epsilon = size0.max(size1) * solver.relative_epsilon();

            assert!(rect0.is_comparable_size(&rect1, epsilon));
        }
    }

    #[test]
    fn skips_non_intersecting_pairs() {
        let quad0 = QuadSegment {
            control_points: [[0.0, 0.0].into(), [50.0, 100.0].into(), [100.0, 0.0].into()],
        };
        let quad1 = QuadSegment {
            control_points: [[200.0, 0.0].into(), [250.0, 100.0].into(), [300.0, 0.0].into()],
        };

        let mut solver: Solver<f64> = Solver::with_grid_size_and_options(0.001, 1.0e-10, 0.001);
        solver.collect_comparable_quad_quad_pairs(quad0, quad1);

        assert!(solver.quad_quad_pairs.is_empty());
    }
}
