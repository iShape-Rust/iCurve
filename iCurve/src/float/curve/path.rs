use crate::float::curve::arc::is_finite_point;
use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Clone, PartialEq)]
pub struct CurvePath<P: FloatPointCompatible> {
    pub(crate) start: P,
    pub(crate) segments: Vec<CurveSegment<P>>,
}

impl<P: FloatPointCompatible> CurvePath<P> {
    #[inline]
    pub fn start(&self) -> P {
        self.start
    }

    #[inline]
    pub fn segments(&self) -> &[CurveSegment<P>] {
        &self.segments
    }

    #[inline]
    pub fn end_point(&self) -> Option<P> {
        self.segments.last().map(CurveSegment::end_point)
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.end_point()
            .is_some_and(|end| end.x() == self.start.x() && end.y() == self.start.y())
    }

    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        let mut bounds = None;
        add_point(&mut bounds, self.start);

        for segment in &self.segments {
            match segment {
                CurveSegment::Line { to } => add_point(&mut bounds, *to),
                CurveSegment::Quad { ctrl, to } => {
                    add_point(&mut bounds, *ctrl);
                    add_point(&mut bounds, *to);
                }
                CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                    add_point(&mut bounds, *ctrl0);
                    add_point(&mut bounds, *ctrl1);
                    add_point(&mut bounds, *to);
                }
                CurveSegment::Arc { arc } => {
                    let ellipse_bounds = arc.ellipse.bounds();
                    bounds = Some(match bounds {
                        Some(bounds) => FloatRect::with_rects(bounds, ellipse_bounds),
                        None => ellipse_bounds,
                    });
                    for point in arc.control_points {
                        add_point(&mut bounds, point);
                    }
                }
            }
        }

        bounds.unwrap_or_else(FloatRect::zero)
    }
}

#[inline]
fn add_point<P: FloatPointCompatible>(bounds: &mut Option<FloatRect<P::Scalar>>, point: P) {
    debug_assert!(is_finite_point(point));
    FloatRect::optional_add_point(bounds, &point);
}
