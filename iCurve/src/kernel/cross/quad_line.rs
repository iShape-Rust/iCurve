use crate::kernel::cross::point::CrossPoint;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::quad::QuadSegment;
use crate::math::quadratic_equation::QuadraticEquation;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_quad_and_line(
        &mut self,
        quad: QuadSegment<T>,
        line: LineSegment<T>,
        output: &mut Vec<CrossPoint<T>>,
    ) {
        // We need to find points where the quadratic Bezier segment lies on the
        // same finite line segment.
        //
        // Quad:
        //   P(t) = (1 - t)^2 * P0 + 2 * (1 - t) * t * P1 + t^2 * P2
        //   t in [0, 1]
        //
        // Line:
        //   L(u) = A + u * D
        //   D = B - A
        //   u in [0, 1]
        //
        // A point P(t) is on the infinite line through A and B when the vector
        // from A to P(t) is parallel to the line direction D. In 2D this is:
        //
        //   cross(P(t) - A, D) = 0
        //
        // Convert the quad from Bernstein form to power form:
        //
        //   P(t) = C2 * t^2 + C1 * t + C0
        //   C0 = P0
        //   C1 = 2 * (P1 - P0)
        //   C2 = P2 - 2 * P1 + P0
        //
        // Substitute it into the line condition:
        //
        //   cross(C2 * t^2 + C1 * t + C0 - A, D) = 0
        //
        // This gives a quadratic equation:
        //
        //   qa * t^2 + qb * t + qc = 0
        //
        // where:
        //
        //   qa = cross(C2, D)
        //   qb = cross(C1, D)
        //   qc = cross(C0 - A, D)
        //
        // Solve this equation for t, keep only roots in [0, 1], then compute
        // the actual point P(t). After that, compute the line parameter u and
        // keep only points where u is also in [0, 1]. Those are the real
        // intersections of the quad segment and the finite line segment.
        //
        // The caller guarantees that the line is not degenerate and that the
        // quad is neither degenerate nor a straight line; these invariants are
        // checked with debug assertions below. A tangent intersection can still
        // produce the same root twice, so duplicate roots are filtered.
        let [p0, p1, p2] = quad.control_points;
        let [a, b] = line.control_points;

        let d = b - a;
        let line_sqr_len = d.sqr_length();

        debug_assert!(
            line_sqr_len != T::ZERO,
            "degenerate line segment is not supported"
        );
        debug_assert!(
            (p1 - p0).cross_product(p2 - p0) != T::ZERO,
            "degenerate or straight quad segment is not supported"
        );

        let quad_rect = quad.to_rect();
        let line_rect =
            FloatRect::with_points(&line.control_points).expect("line segment has control points");

        if !quad_rect.is_intersect_with_padding(&line_rect, self.grid_size()) {
            return;
        }

        let c0 = p0;
        let c1 = (p1 - p0) * T::TWO;
        let c2 = p2 - p1 * T::TWO + p0;

        let qa = c2.cross_product(d);
        let qb = c1.cross_product(d);
        let qc = (c0 - a).cross_product(d);

        debug_assert!(
            qa != T::ZERO || qb != T::ZERO || qc != T::ZERO,
            "collinear quad-line overlap is not supported"
        );

        let Some(roots) = QuadraticEquation::solve(qa, qb, qc) else {
            return;
        };

        self.push_quad_line_root(roots[0], a, d, line_sqr_len, c0, c1, c2, output);

        if (roots[1] - roots[0]).abs() > self.grid_size() {
            self.push_quad_line_root(roots[1], a, d, line_sqr_len, c0, c1, c2, output);
        }
    }

    fn push_quad_line_root(
        &self,
        t: T,
        line_start: FloatPoint<T>,
        line_dir: FloatPoint<T>,
        line_sqr_len: T,
        c0: FloatPoint<T>,
        c1: FloatPoint<T>,
        c2: FloatPoint<T>,
        output: &mut Vec<CrossPoint<T>>,
    ) {
        if !SegmentParam::is_in_unit_range(t, self.grid_size()) {
            return;
        }

        let t = SegmentParam::clamp_unit(t);
        let point = c2 * (t * t) + c1 * t + c0;
        let u = (point - line_start).dot_product(line_dir) / line_sqr_len;

        if !SegmentParam::is_in_unit_range(u, self.grid_size()) {
            return;
        }

        let u = SegmentParam::clamp_unit(u);

        output.push(CrossPoint {
            point,
            t0: SegmentParam::from(t),
            t1: SegmentParam::from(u),
        });
    }
}
