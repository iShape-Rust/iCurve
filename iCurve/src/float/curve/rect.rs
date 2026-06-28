use crate::float::curve::path::CurvePath;
use crate::float::curve::segment::CurveSegment;
use crate::float::curve::shape::CurveShape;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

pub trait CurveToFloatRect<T: FloatNumber> {
    fn float_rect(&self) -> Option<FloatRect<T>>;
}

impl<P: FloatPointCompatible<Scalar = T>, T: FloatNumber> CurveToFloatRect<T> for CurveShape<P> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for contour in &self.contours {
            rect = FloatRect::with_optional_rects(rect, contour.float_rect());
        }
        rect
    }
}

impl<P: FloatPointCompatible<Scalar = T>, T: FloatNumber> CurveToFloatRect<T> for CurvePath<P> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        FloatRect::optional_add_point(&mut rect, &self.start);
        for segment in &self.segments {
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
            }
        }
        rect
    }
}

impl<P: FloatPointCompatible<Scalar = T>, T: FloatNumber> CurveToFloatRect<T> for [CurveShape<P>] {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for shape in self {
            rect = FloatRect::with_optional_rects(rect, shape.float_rect());
        }
        rect
    }
}
