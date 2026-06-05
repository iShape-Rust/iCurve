use super::{CurveSpan, close_parameter, close_point, point_at};
use crate::flatten::segment::{LineSegment, SegmentParam};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(super) fn can_recombine<P, I>(
    prev: CurveSpan<P, I>,
    next: CurveSpan<P, I>,
    line: &LineSegment<P>,
    next_line: &LineSegment<P>,
    adapter: &FloatPointAdapter<P, I>,
) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    let v0 = prev.end - prev.start;
    let v1 = next.end - prev.end;
    let p0 = adapter.int_to_float(&prev.start);
    let p2 = adapter.int_to_float(&next.end);

    v0.cross_product(v1) == I::Wide::ZERO
        && close_parameter(prev.range.t1.value(), next.range.t0.value())
        && close_point(line_point_at(line, next.range.t1), p2, adapter)
        && close_point(line_point_at(next_line, prev.range.t0), p0, adapter)
}

fn line_point_at<P: FloatPointCompatible>(line: &LineSegment<P>, t: SegmentParam<P::Scalar>) -> P {
    match t {
        SegmentParam::Start => line.control_points[0],
        SegmentParam::Inner(t) => point_at(line.control_points[0], line.control_points[1], t),
        SegmentParam::End => line.control_points[1],
    }
}

#[cfg(test)]
mod tests {
    use super::super::CurveSpan;
    use crate::bool::overlay::CurveOverlay;
    use crate::bool::scale::FixedScaleCurveOverlay;
    use crate::curve::arc::EllipticArc;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};
    use crate::flatten::segment::{LineSegment, NormalizedSegment, SegmentRange};
    use crate::util::adapter::TestAdapter;
    use i_overlay::i_float::adapter::FloatPointAdapter;

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
        let segment = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0], [4.0, 0.0]],
        });

        let s0 = span(
            [0.0, 0.0],
            [2.0, 0.0],
            &segment,
            SegmentRange::new(0, 0.0, 0.5),
            &adapter,
        );

        let s1 = span(
            [2.0, 0.0],
            [4.0, 0.0],
            &segment,
            SegmentRange::new(0, 0.5, 1.0),
            &adapter,
        );

        assert!(s0.can_recombine_with(s1, &adapter));
    }

    #[test]
    fn test_1() {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);
        let a = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0], [2.0, 0.0]],
        });
        let b = NormalizedSegment::Line(LineSegment {
            control_points: [[2.0, 0.0], [4.0, 1.0]],
        });

        assert!(
            !span(
                [0.0, 0.0],
                [2.0, 0.0],
                &a,
                SegmentRange::new(0, 0.0, 0.5),
                &adapter
            )
            .can_recombine_with(
                span(
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
    fn test_2() -> Result<(), CurveError> {
        let adapter = FloatPointAdapter::with_radius_and_scale(10.0, 1000.0);

        let square_0 = CurveShapeBuilder::new()
            .move_to([-10.0, 0.0])?
            .line_to([0.0, 0.0])?
            .line_to([0.0, 10.0])?
            .line_to([-10.0, 10.0])?
            .line_to([-10.0, 0.0])?
            .close()?
            .build()?;

        let square_1 = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([10.0, 0.0])?
            .line_to([10.0, 10.0])?
            .line_to([0.0, 10.0])?
            .line_to([0.0, 0.0])?
            .close()?
            .build()?;

        let result = square_0.overlay_with_fixed_scale(&square_1);

        Ok(())
    }
}
