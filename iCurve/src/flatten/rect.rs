use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::convert::ArcEndPoint;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

pub(crate) trait ShapeFloatRect<P: FloatPointCompatible> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>>;
}

impl<P: FloatPointCompatible> ShapeFloatRect<P> for CurveShape<P> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for contour in &self.contours {
            FloatRect::optional_add_point(&mut rect, &contour.start);
            for segment in &contour.segments {
                match *segment {
                    CurveSegment::Line { to } => FloatRect::optional_add_point(&mut rect, &to),
                    CurveSegment::Quad { ctrl, to } => {
                        FloatRect::optional_add_point(&mut rect, &ctrl);
                        FloatRect::optional_add_point(&mut rect, &to);
                    }
                    CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                        FloatRect::optional_add_point(&mut rect, &ctrl0);
                        FloatRect::optional_add_point(&mut rect, &ctrl1);
                        FloatRect::optional_add_point(&mut rect, &to);
                    }
                    CurveSegment::Arc { ref arc } => {
                        FloatRect::optional_add_point(&mut rect, &arc.center);
                        FloatRect::optional_add_point(&mut rect, &arc.end_point());
                    }
                }
            }
        }
        rect
    }
}

impl<P: FloatPointCompatible> ShapeFloatRect<P> for [CurveShape<P>] {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for shape in self {
            rect = FloatRect::with_optional_rects(rect, shape.float_rect());
        }
        rect
    }
}

