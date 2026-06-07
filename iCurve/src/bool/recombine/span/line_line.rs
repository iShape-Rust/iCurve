use super::{CurveSpan, point_at};
use crate::flatten::segment::{LineSegment, SegmentParam};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(super) fn can_recombine<P, I>(
    prev: CurveSpan<P, I>,
    next: CurveSpan<P, I>,
) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    debug_assert!(next.start == prev.end);

    let v0 = prev.end - prev.start;
    let v1 = next.end - prev.end;

    v0.cross_product(v1) == I::Wide::ZERO
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
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::{ShapeType};
    use i_overlay::core::overlay_rule::OverlayRule;
    use super::super::CurveSpan;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};
    use crate::flatten::segment::{LineSegment, NormalizedSegment, SegmentRange};
    use crate::util::adapter::TestAdapter;
    use i_overlay::i_float::adapter::FloatPointAdapter;
    use crate::bool::overlay::CurveOverlay;

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

        let mut overlay: CurveOverlay<_, i32> = CurveOverlay::with_adapter(FloatPointAdapter::with_radius_and_scale(100.0, 1000.0));

        _ = overlay.add_shape(&square_0, ShapeType::Subject);
        _ = overlay.add_shape(&square_1, ShapeType::Clip);

        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);

        debug_assert_eq!(result.len(), 1);

        Ok(())
    }

    #[test]
    fn test_3() -> Result<(), CurveError> {
        let square_0 = CurveShapeBuilder::new()
            .move_to([-10.0, 0.0])?
            .line_to([0.0, 0.0])?
            .line_to([0.0, 10.0])?
            .line_to([-10.0, 10.0])?
            .close_with_line()?
            .build()?;

        let square_1 = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([0.0, 10.0])?
            .line_to([10.0, 10.0])?
            .line_to([10.0, 0.0])?
            .close_with_line()?
            .build()?;

        let mut overlay: CurveOverlay<_, i32> = CurveOverlay::with_adapter(FloatPointAdapter::with_radius_and_scale(100.0, 1000.0));

        _ =overlay.add_shape(&square_0, ShapeType::Subject);
        _ =overlay.add_shape(&square_1, ShapeType::Clip);

        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);

        debug_assert_eq!(result.len(), 1);

        Ok(())
    }
}
