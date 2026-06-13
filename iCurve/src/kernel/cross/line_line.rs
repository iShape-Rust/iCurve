use crate::kernel::cross::point::CrossPoint;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::param::SegmentParam;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_line_and_line(
        &mut self,
        line0: LineSegment<T>,
        line1: LineSegment<T>,
        output: &mut Vec<CrossPoint<T>>,
    ) {
        // Line segments are represented in parametric form:
        //
        //   L0(t) = A + t * R,  t in [0, 1]
        //   L1(u) = B + u * S,  u in [0, 1]
        //
        // At the intersection:
        //
        //   A + t * R = B + u * S
        //
        // Using cross products removes one unknown at a time:
        //
        //   t = cross(B - A, S) / cross(R, S)
        //   u = cross(B - A, R) / cross(R, S)
        //
        // When cross(R, S) is zero, the lines are parallel. Non-collinear
        // parallel segments do not intersect. Collinear segments may overlap;
        // in that case we report the overlap boundaries as cross points.
        let [a0, a1] = line0.control_points;
        let [b0, b1] = line1.control_points;

        let r = a1 - a0;
        let s = b1 - b0;
        let r_sqr_len = r.sqr_length();
        let s_sqr_len = s.sqr_length();

        debug_assert!(r_sqr_len != T::ZERO, "degenerate line segment is not supported");
        debug_assert!(s_sqr_len != T::ZERO, "degenerate line segment is not supported");

        let line0_rect =
            FloatRect::with_points(&line0.control_points).expect("line segment has control points");
        let line1_rect =
            FloatRect::with_points(&line1.control_points).expect("line segment has control points");

        if !line0_rect.is_intersect_with_padding(&line1_rect, self.grid_size()) {
            return;
        }

        let b_to_a = b0 - a0;
        let denom = r.cross_product(s);

        if denom == T::ZERO {
            if b_to_a.cross_product(r) != T::ZERO {
                return;
            }

            self.push_collinear_line_line(a0, r, r_sqr_len, b0, s, s_sqr_len, output);
            return;
        }

        let t = b_to_a.cross_product(s) / denom;
        let u = b_to_a.cross_product(r) / denom;

        if !SegmentParam::is_in_unit_range(t, self.grid_size())
            || !SegmentParam::is_in_unit_range(u, self.grid_size())
        {
            return;
        }

        let t = SegmentParam::clamp_unit(t);
        let u = SegmentParam::clamp_unit(u);

        output.push(CrossPoint {
            point: a0 + r * t,
            t0: SegmentParam::from(t),
            t1: SegmentParam::from(u),
        });
    }

    fn push_collinear_line_line(
        &self,
        a0: FloatPoint<T>,
        r: FloatPoint<T>,
        r_sqr_len: T,
        b0: FloatPoint<T>,
        s: FloatPoint<T>,
        s_sqr_len: T,
        output: &mut Vec<CrossPoint<T>>,
    ) {
        let t0 = (b0 - a0).dot_product(r) / r_sqr_len;
        let t1 = (b0 + s - a0).dot_product(r) / r_sqr_len;

        let min_t = t0.min(t1).max(T::ZERO);
        let max_t = t0.max(t1).min(T::ONE);

        if min_t > max_t + self.grid_size() {
            return;
        }

        self.push_collinear_line_line_point(min_t, a0, r, b0, s, s_sqr_len, output);

        if max_t - min_t > self.grid_size() {
            self.push_collinear_line_line_point(max_t, a0, r, b0, s, s_sqr_len, output);
        }
    }

    fn push_collinear_line_line_point(
        &self,
        t: T,
        a0: FloatPoint<T>,
        r: FloatPoint<T>,
        b0: FloatPoint<T>,
        s: FloatPoint<T>,
        s_sqr_len: T,
        output: &mut Vec<CrossPoint<T>>,
    ) {
        let t = SegmentParam::clamp_unit(t);
        let point = a0 + r * t;
        let u = (point - b0).dot_product(s) / s_sqr_len;
        let u = SegmentParam::clamp_unit(u);

        output.push(CrossPoint {
            point,
            t0: SegmentParam::from(t),
            t1: SegmentParam::from(u),
        });
    }
}
