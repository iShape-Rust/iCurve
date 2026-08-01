use crate::float::curve::arc::is_finite_point;
use crate::float::curve::path::CurvePath;
use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Clone, PartialEq)]
pub struct CurveShape<P: FloatPointCompatible> {
    pub(crate) contours: Vec<CurvePath<P>>,
}

impl<P: FloatPointCompatible> CurveShape<P> {
    #[inline]
    pub fn contours(&self) -> &[CurvePath<P>] {
        &self.contours
    }

    /// Returns the total number of curve segments in the shape.
    pub fn segment_count(&self) -> usize {
        self.contours.iter().map(|path| path.segments.len()).sum()
    }

    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        let mut bounds = None;

        for contour in &self.contours {
            add_point(&mut bounds, contour.start);
            for segment in &contour.segments {
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
        }

        bounds.unwrap_or_else(FloatRect::zero)
    }
}

#[inline]
fn add_point<P: FloatPointCompatible>(bounds: &mut Option<FloatRect<P::Scalar>>, point: P) {
    debug_assert!(is_finite_point(point));
    FloatRect::optional_add_point(bounds, &point);
}
