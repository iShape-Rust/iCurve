use crate::flatten::segment::{LineSegment, NormalizedSegment, QuadSegment, SegmentParam, SegmentRange};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;

#[derive(Clone, Copy)]
pub(super) struct CurveGeometry<'a, P: FloatPointCompatible, I: IntNumber> {
    pub(super) start: IntPoint<I>,
    pub(super) end: IntPoint<I>,
    pub(super) segment: &'a NormalizedSegment<P>,
    pub(super) range: SegmentRange<P::Scalar>,
}

impl<'a, P, I> CurveGeometry<'a, P, I>
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    #[inline(always)]
    pub(super) fn new(
        start: IntPoint<I>,
        end: IntPoint<I>,
        segment: &'a NormalizedSegment<P>,
        range: SegmentRange<P::Scalar>,
    ) -> Self {
        Self {
            start,
            end,
            segment,
            range,
        }
    }

    pub(super) fn compare(self, next: Self, adapter: &FloatPointAdapter<P, I>) -> bool {
        debug_assert!(self.end == next.start);

        match (self.segment, next.segment) {
            (NormalizedSegment::Line(a), NormalizedSegment::Line(b)) => {
                self.compare_lines(next, a, b, adapter)
            }
            (NormalizedSegment::Quad(a), NormalizedSegment::Quad(b)) => {
                self.compare_quads(next, a, b, adapter)
            }
            _ => false,
        }
    }

    fn compare_lines(
        self,
        next: Self,
        line: &LineSegment<P>,
        next_line: &LineSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> bool {
        let v0 = self.end - self.start;
        let v1 = next.end - self.end;
        let p0 = adapter.int_to_float(&self.start);
        let p2 = adapter.int_to_float(&next.end);

        v0.cross_product(v1) == I::Wide::ZERO
            && close_parameter(self.range.t1.value(), next.range.t0.value())
            && close_point(line_point_at(line, next.range.t1), p2, adapter)
            && close_point(line_point_at(next_line, self.range.t0), p0, adapter)
    }

    fn compare_quads(
        self,
        next: Self,
        full_quad: &QuadSegment<P>,
        full_next_quad: &QuadSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> bool {
        let Some(quad) = full_quad.range(self.range.t0, self.range.t1) else {
            return false;
        };
        let Some(next_quad) = full_next_quad.range(next.range.t0, next.range.t1) else {
            return false;
        };

        let p0 = adapter.int_to_float(&self.start);
        let c0 = quad.control_points[1];
        let p1 = adapter.int_to_float(&self.end);
        let c1 = next_quad.control_points[1];
        let p2 = adapter.int_to_float(&next.end);

        if !same_tangent(c0, self.end, c1, adapter) {
            return false;
        }

        let Some(t) = quad_split_parameter(c0, p1, c1) else {
            return false;
        };
        let Some(expected_t) = range_split_parameter(self.range, next.range) else {
            return false;
        };

        let one = P::Scalar::from_float(1.0);
        let q0 = interpolate_outer_control_from_left(p0, c0, t);
        let q1 = interpolate_outer_control_from_right(c1, p2, t);
        let control = midpoint(q0, q1);

        if !close_point(q0, q1, adapter) {
            return false;
        }

        let [left, right] = split_quad(
            QuadSegment {
                control_points: [p0, control, p2],
            },
            t,
        );

        close_parameter(t, expected_t)
            && close_point(quad_point_at(full_quad, next.range.t1), p2, adapter)
            && close_point(quad_point_at(full_next_quad, self.range.t0), p0, adapter)
            && close_point(left.control_points[0], p0, adapter)
            && close_point(left.control_points[1], c0, adapter)
            && close_point(left.control_points[2], p1, adapter)
            && close_point(right.control_points[0], p1, adapter)
            && close_point(right.control_points[1], c1, adapter)
            && close_point(right.control_points[2], p2, adapter)
            && t > P::Scalar::from_float(0.0)
            && t < one
    }
}

fn same_tangent<P, I>(c0: P, join: IntPoint<I>, c1: P, adapter: &FloatPointAdapter<P, I>) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    let c0 = adapter.float_to_int(&c0);
    let c1 = adapter.float_to_int(&c1);
    let v0 = join - c0;
    let v1 = c1 - join;

    v0.cross_product(v1) == I::Wide::ZERO && v0.dot_product(v1) > I::Wide::ZERO
}

fn quad_split_parameter<P: FloatPointCompatible>(c0: P, join: P, c1: P) -> Option<P::Scalar> {
    let a = length(vector(c0, join));
    let b = length(vector(join, c1));
    let sum = a + b;

    if sum == P::Scalar::from_float(0.0) {
        None
    } else {
        Some(a / sum)
    }
}

fn range_split_parameter<F: FloatNumber>(prev: SegmentRange<F>, next: SegmentRange<F>) -> Option<F> {
    let t0 = prev.t0.value();
    let t1 = prev.t1.value();
    let t2 = next.t1.value();
    let denom = t2 - t0;

    if denom == F::from_float(0.0) {
        None
    } else {
        Some((t1 - t0) / denom)
    }
}

fn close_parameter<F: FloatNumber>(a: F, b: F) -> bool {
    (a - b).abs() <= F::from_float(0.0001)
}

fn interpolate_outer_control_from_left<P: FloatPointCompatible>(p0: P, c0: P, t: P::Scalar) -> P {
    let one = P::Scalar::from_float(1.0);
    let inv_t = one / t;

    P::from_xy(
        (c0.x() - p0.x() * (one - t)) * inv_t,
        (c0.y() - p0.y() * (one - t)) * inv_t,
    )
}

fn interpolate_outer_control_from_right<P: FloatPointCompatible>(c1: P, p2: P, t: P::Scalar) -> P {
    let one = P::Scalar::from_float(1.0);
    let inv = one / (one - t);

    P::from_xy((c1.x() - p2.x() * t) * inv, (c1.y() - p2.y() * t) * inv)
}

fn line_point_at<P: FloatPointCompatible>(line: &LineSegment<P>, t: SegmentParam<P::Scalar>) -> P {
    match t {
        SegmentParam::Start => line.control_points[0],
        SegmentParam::Inner(t) => point_at(line.control_points[0], line.control_points[1], t),
        SegmentParam::End => line.control_points[1],
    }
}

fn quad_point_at<P: FloatPointCompatible>(quad: &QuadSegment<P>, t: SegmentParam<P::Scalar>) -> P {
    match t {
        SegmentParam::Start => quad.control_points[0],
        SegmentParam::Inner(t) => {
            let [left, _] = split_quad(*quad, t);
            left.control_points[2]
        }
        SegmentParam::End => quad.control_points[2],
    }
}

fn split_quad<P: FloatPointCompatible>(quad: QuadSegment<P>, t: P::Scalar) -> [QuadSegment<P>; 2] {
    let [p0, p1, p2] = quad.control_points;
    let p01 = point_at(p0, p1, t);
    let p12 = point_at(p1, p2, t);
    let p012 = point_at(p01, p12, t);

    [
        QuadSegment {
            control_points: [p0, p01, p012],
        },
        QuadSegment {
            control_points: [p012, p12, p2],
        },
    ]
}

fn point_at<P: FloatPointCompatible>(a: P, b: P, t: P::Scalar) -> P {
    P::from_xy(a.x() + (b.x() - a.x()) * t, a.y() + (b.y() - a.y()) * t)
}

fn midpoint<P: FloatPointCompatible>(a: P, b: P) -> P {
    let half = P::Scalar::from_float(0.5);
    P::from_xy((a.x() + b.x()) * half, (a.y() + b.y()) * half)
}

fn vector<P: FloatPointCompatible>(a: P, b: P) -> P {
    P::from_xy(b.x() - a.x(), b.y() - a.y())
}

fn length<P: FloatPointCompatible>(v: P) -> P::Scalar {
    (v.x() * v.x() + v.y() * v.y()).sqrt()
}

fn close_point<P, I>(a: P, b: P, adapter: &FloatPointAdapter<P, I>) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let sqr_distance = dx * dx + dy * dy;

    adapter.round_sqr_len_to_int(sqr_distance) <= I::Wide::ONE
}

trait QuadRange<P: FloatPointCompatible> {
    fn range(&self, t0: SegmentParam<P::Scalar>, t1: SegmentParam<P::Scalar>) -> Option<Self>
    where
        Self: Sized;
}

impl<P: FloatPointCompatible> QuadRange<P> for QuadSegment<P> {
    fn range(&self, t0: SegmentParam<P::Scalar>, t1: SegmentParam<P::Scalar>) -> Option<Self> {
        if t0 == t1 {
            return None;
        }

        if t0.value() < t1.value() {
            Some(forward_quad_range(*self, t0, t1))
        } else {
            Some(reverse_quad(forward_quad_range(*self, t1, t0)))
        }
    }
}

fn forward_quad_range<P: FloatPointCompatible>(
    quad: QuadSegment<P>,
    t0: SegmentParam<P::Scalar>,
    t1: SegmentParam<P::Scalar>,
) -> QuadSegment<P> {
    if t0 == SegmentParam::Start && t1 == SegmentParam::End {
        return quad;
    }

    if t0 == SegmentParam::Start {
        let [segment, _] = split_quad(quad, t1.value());
        return segment;
    }

    let [_, right] = split_quad(quad, t0.value());
    if t1 == SegmentParam::End {
        return right;
    }

    let one = P::Scalar::from_float(1.0);
    let t0 = t0.value();
    let local_t = (t1.value() - t0) / (one - t0);
    let [segment, _] = split_quad(right, local_t);
    segment
}

fn reverse_quad<P: FloatPointCompatible>(quad: QuadSegment<P>) -> QuadSegment<P> {
    let [p0, p1, p2] = quad.control_points;
    QuadSegment {
        control_points: [p2, p1, p0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::segment::{LineSegment, QuadSegment};
    use i_overlay::i_float::float::rect::FloatRect;

    fn adapter() -> FloatPointAdapter<[f64; 2], i32> {
        FloatPointAdapter::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0)
    }

    fn curve<'a>(
        start: [f64; 2],
        end: [f64; 2],
        segment: &'a NormalizedSegment<[f64; 2]>,
        range: SegmentRange<f64>,
        adapter: &FloatPointAdapter<[f64; 2], i32>,
    ) -> CurveGeometry<'a, [f64; 2], i32> {
        CurveGeometry::new(
            adapter.float_to_int(&start),
            adapter.float_to_int(&end),
            segment,
            range,
        )
    }

    #[test]
    fn adjacent_lines_match_when_collinear_and_same_direction() {
        let adapter = adapter();
        let segment = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0], [4.0, 0.0]],
        });

        assert!(
            curve(
                [0.0, 0.0],
                [2.0, 0.0],
                &segment,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .compare(
                curve(
                    [2.0, 0.0],
                    [4.0, 0.0],
                    &segment,
                    SegmentRange::new(0, 0.5, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }

    #[test]
    fn adjacent_lines_do_not_match_when_not_collinear() {
        let adapter = adapter();
        let a = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0], [2.0, 0.0]],
        });
        let b = NormalizedSegment::Line(LineSegment {
            control_points: [[2.0, 0.0], [4.0, 1.0]],
        });

        assert!(
            !curve(
                [0.0, 0.0],
                [2.0, 0.0],
                &a,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .compare(
                curve(
                    [2.0, 0.0],
                    [4.0, 1.0],
                    &b,
                    SegmentRange::new(1, 0.5, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }

    #[test]
    fn adjacent_quad_ranges_match_when_they_are_split_from_one_quad() {
        let adapter = adapter();
        let source = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 4.0], [6.0, 0.0]],
        };
        let segment = NormalizedSegment::Quad(source);

        assert!(
            curve(
                [0.0, 0.0],
                [2.5, 2.0],
                &segment,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .compare(
                curve(
                    [2.5, 2.0],
                    [6.0, 0.0],
                    &segment,
                    SegmentRange::new(0, 0.5, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }

    #[test]
    fn adjacent_quad_ranges_match_with_non_half_split() {
        let adapter = adapter();
        let source = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 4.0], [6.0, 0.0]],
        };
        let segment = NormalizedSegment::Quad(source);

        assert!(
            curve(
                [0.0, 0.0],
                [1.125, 1.5],
                &segment,
                SegmentRange::new(0, 0.0, 0.25),
                &adapter
            )
            .compare(
                curve(
                    [1.125, 1.5],
                    [6.0, 0.0],
                    &segment,
                    SegmentRange::new(0, 0.25, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }

    #[test]
    fn adjacent_quads_do_not_match_when_tangent_is_broken() {
        let adapter = adapter();
        let left = NormalizedSegment::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [1.0, 2.0], [2.0, 2.0]],
        });
        let right = NormalizedSegment::Quad(QuadSegment {
            control_points: [[2.0, 2.0], [3.0, 3.0], [6.0, 0.0]],
        });

        assert!(
            !curve(
                [0.0, 0.0],
                [2.0, 2.0],
                &left,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .compare(
                curve(
                    [2.0, 2.0],
                    [6.0, 0.0],
                    &right,
                    SegmentRange::new(1, 0.5, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }

    #[test]
    fn adjacent_quads_do_not_match_when_they_do_not_reconstruct_one_quad() {
        let adapter = adapter();
        let left = NormalizedSegment::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [1.0, 2.0], [2.0, 2.0]],
        });
        let right = NormalizedSegment::Quad(QuadSegment {
            control_points: [[2.0, 2.0], [3.0, 2.0], [6.0, 4.0]],
        });

        assert!(
            !curve(
                [0.0, 0.0],
                [2.0, 2.0],
                &left,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .compare(
                curve(
                    [2.0, 2.0],
                    [6.0, 4.0],
                    &right,
                    SegmentRange::new(1, 0.5, 1.0),
                    &adapter
                ),
                &adapter
            )
        );
    }
}
