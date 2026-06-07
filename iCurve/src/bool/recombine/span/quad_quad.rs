use super::{CurveSpan, close_parameter, close_point, point_at, range_split_parameter};
use crate::flatten::segment::{QuadSegment, SegmentParam};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;

pub(super) fn can_recombine<P, I>(
    prev: CurveSpan<P, I>,
    next: CurveSpan<P, I>,
    full_prev_quad: &QuadSegment<P>,
    full_next_quad: &QuadSegment<P>,
    adapter: &FloatPointAdapter<P, I>,
) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    debug_assert!(prev.end == next.start);

    let Some(prev_quad) = full_prev_quad.range(prev.range.t0, prev.range.t1) else {
        return false;
    };
    let Some(next_quad) = full_next_quad.range(next.range.t0, next.range.t1) else {
        return false;
    };

    let p0 = adapter.int_to_float(&prev.start);
    let c0 = prev_quad.control_points[1];
    let p1 = adapter.int_to_float(&prev.end);
    let c1 = next_quad.control_points[1];
    let p2 = adapter.int_to_float(&next.end);

    if !same_tangent(c0, prev.end, c1, adapter) {
        return false;
    }

    let Some(t) = quad_split_parameter(c0, p1, c1) else {
        return false;
    };
    let Some(expected_t) = range_split_parameter(prev.range, next.range) else {
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
        && close_point(quad_point_at(full_prev_quad, next.range.t1), p2, adapter)
        && close_point(quad_point_at(full_next_quad, prev.range.t0), p0, adapter)
        && close_point(left.control_points[0], p0, adapter)
        && close_point(left.control_points[1], c0, adapter)
        && close_point(left.control_points[2], p1, adapter)
        && close_point(right.control_points[0], p1, adapter)
        && close_point(right.control_points[1], c1, adapter)
        && close_point(right.control_points[2], p2, adapter)
        && t > P::Scalar::from_float(0.0)
        && t < one
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
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::core::overlay_rule::OverlayRule;
    use super::super::CurveSpan;
    use crate::flatten::segment::{NormalizedSegment, QuadSegment, SegmentRange};
    use i_overlay::i_float::adapter::FloatPointAdapter;
    use crate::bool::overlay::CurveOverlay;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};
    use crate::flatten::split::SplitAt;
    use crate::util::adapter::TestAdapter;

    fn span<'a>(
        start: [f64; 2],
        end: [f64; 2],
        segment: &'a NormalizedSegment<[f64; 2]>,
        range: SegmentRange<f64>,
        adapter: &FloatPointAdapter<[f64; 2], i32>,
    ) -> CurveSpan<'a, [f64; 2], i32> {
        CurveSpan::new(
            adapter.float_to_int(&start),
            adapter.float_to_int(&end),
            segment,
            range,
        )
    }

    #[test]
    fn test_0() {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);
        let source = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 4.0], [6.0, 0.0]],
        };
        let segment = NormalizedSegment::Quad(source);

        assert!(
            span(
                [0.0, 0.0],
                [2.5, 2.0],
                &segment,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .can_recombine_with(
                span(
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
    fn test_1() {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);
        let source = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 4.0], [6.0, 0.0]],
        };
        let segment = NormalizedSegment::Quad(source);

        assert!(
            span(
                [0.0, 0.0],
                [1.125, 1.5],
                &segment,
                SegmentRange::new(0, 0.0, 0.25),
                &adapter
            )
            .can_recombine_with(
                span(
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
    fn test_2() {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);
        let left = NormalizedSegment::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [1.0, 2.0], [2.0, 2.0]],
        });
        let right = NormalizedSegment::Quad(QuadSegment {
            control_points: [[2.0, 2.0], [3.0, 3.0], [6.0, 0.0]],
        });

        assert!(
            !span(
                [0.0, 0.0],
                [2.0, 2.0],
                &left,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .can_recombine_with(
                span(
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
    fn test_3() {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);
        let left = NormalizedSegment::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [1.0, 2.0], [2.0, 2.0]],
        });
        let right = NormalizedSegment::Quad(QuadSegment {
            control_points: [[2.0, 2.0], [3.0, 2.0], [6.0, 4.0]],
        });

        assert!(
            !span(
                [0.0, 0.0],
                [2.0, 2.0],
                &left,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .can_recombine_with(
                span(
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

    #[test]
    fn test_4() -> Result<(), CurveError> {
        let [q0, q1] = QuadSegment { control_points: [[4.0, 0.0], [0.0, 4.0], [-4.0, 0.0]] }.split_at(0.5);

        let shape_0 = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to(q0.control_points[0])?
            .quad_to(q0.control_points[1], q0.control_points[2])?
            .close_with_line()?
            .build()?;

        let shape_1 = CurveShapeBuilder::new()
            .move_to(q1.control_points[0])?
            .quad_to(q1.control_points[1], q1.control_points[2])?
            .line_to([0.0, 0.0])?
            .close_with_line()?
            .build()?;

        let mut overlay: CurveOverlay<_, i32> = CurveOverlay::with_adapter(FloatPointAdapter::with_radius_and_scale(100.0, 1000.0));

        _ =overlay.add_shape(&shape_0, ShapeType::Subject);
        _ =overlay.add_shape(&shape_1, ShapeType::Clip);

        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);

        debug_assert_eq!(result.len(), 1);

        Ok(())
    }
}
