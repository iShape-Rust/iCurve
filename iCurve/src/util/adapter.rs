use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;

pub(crate) trait TestAdapter<P: FloatPointCompatible, I: IntNumber = i32> {
    fn with_radius_and_scale(radius: P::Scalar, scale: P::Scalar) -> FloatPointAdapter<P, I>;
}

impl<P: FloatPointCompatible, I: IntNumber> TestAdapter<P, I> for FloatPointAdapter<P, I> {
    fn with_radius_and_scale(radius: P::Scalar, scale: P::Scalar) -> FloatPointAdapter<P, I> {
        let rect = FloatRect::new(-radius, radius, -radius, radius);
        FloatPointAdapter::with_scale(rect, scale)
    }
}
