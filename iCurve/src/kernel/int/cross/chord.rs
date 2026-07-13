use crate::kernel::int::curve::chord::SegmentChord;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChordCross<I: IntNumber> {
    Point(IntPoint<I>),
    Overlay,
}

impl<I: IntNumber> SegmentChord<I> {
    pub(crate) fn cross(&self, other: &Self, radius: I::Wide) -> Option<ChordCross<I>> {
        let a0b0a1 = Triangle::clock_direction(self.a, self.b, other.a);
        let a0b0b1 = Triangle::clock_direction(self.a, self.b, other.b);
        let a1b1a0 = Triangle::clock_direction(other.a, other.b, self.a);
        let a1b1b0 = Triangle::clock_direction(other.a, other.b, self.b);

        let one = I::Wide::ONE;
        let zero_count =
            (one & (a0b0a1 + one)) + (one & (a0b0b1 + one)) + (one & (a1b1a0 + one)) + (one & (a1b1b0 + one));

        if zero_count == I::Wide::FOUR {
            return Some(ChordCross::Overlay);
        }

        let is_not_cross = a0b0a1 == a0b0b1 || a1b1a0 == a1b1b0;
        if zero_count > I::Wide::ONE || is_not_cross {
            return None;
        }

        if zero_count != I::Wide::ZERO {
            let point = if a0b0a1 == I::Wide::ZERO {
                other.a
            } else if a0b0b1 == I::Wide::ZERO {
                other.b
            } else if a1b1a0 == I::Wide::ZERO {
                self.a
            } else {
                self.b
            };
            return Some(ChordCross::Point(point));
        }

        Some(ChordCross::Point(self.middle_cross_point(other, radius)))
    }

    fn middle_cross_point(&self, other: &Self, radius: I::Wide) -> IntPoint<I> {
        let point = self.cross_point(other);
        if Triangle::is_line(self.a, point, self.b) && Triangle::is_line(other.a, point, other.b) {
            return point;
        }

        let ra0 = self.a.sqr_distance(point);
        let rb0 = self.b.sqr_distance(point);
        let ra1 = other.a.sqr_distance(point);
        let rb1 = other.b.sqr_distance(point);

        if ra0 <= radius || rb0 <= radius || ra1 <= radius || rb1 <= radius {
            let r0 = ra0.min(rb0);
            let r1 = ra1.min(rb1);
            if r0 <= r1 {
                let endpoint = if ra0 < rb0 { self.a } else { self.b };
                if Triangle::is_not_line(other.a, endpoint, other.b) {
                    return endpoint;
                }
            } else {
                let endpoint = if ra1 < rb1 { other.a } else { other.b };
                if Triangle::is_not_line(self.a, endpoint, self.b) {
                    return endpoint;
                }
            }
        }

        point
    }

    fn cross_point(&self, other: &Self) -> IntPoint<I> {
        let a0x = self.a.x.to_wide();
        let a0y = self.a.y.to_wide();
        let a1x = self.b.x.to_wide() - a0x;
        let a1y = self.b.y.to_wide() - a0y;
        let b0x = other.a.x.to_wide() - a0x;
        let b0y = other.a.y.to_wide() - a0y;
        let b1x = other.b.x.to_wide() - a0x;
        let b1y = other.b.y.to_wide() - a0y;

        let dx_b = b0x - b1x;
        let dy_b = b0y - b1y;
        let xy_b = b0x * b1y - b0y * b1x;

        let (x0, y0) = if a1x == I::Wide::ZERO {
            (I::Wide::ZERO, xy_b / dx_b)
        } else if a1y == I::Wide::ZERO {
            (-xy_b / dy_b, I::Wide::ZERO)
        } else {
            let divider = a1y * dx_b - a1x * dy_b;
            let sign = divider.signum() * xy_b.signum();
            let sx = a1x.signum() * sign;
            let sy = a1y.signum() * sign;
            let abs_xy = xy_b.unsigned_abs();
            let abs_divider = divider.unsigned_abs();
            let x_product = <I::WideUInt as UIntNumber>::Product::multiply(a1x.unsigned_abs(), abs_xy);
            let y_product = <I::WideUInt as UIntNumber>::Product::multiply(a1y.unsigned_abs(), abs_xy);
            let x = sx * I::Wide::from_uint(x_product.divide_with_rounding(abs_divider));
            let y = sy * I::Wide::from_uint(y_product.divide_with_rounding(abs_divider));
            (x, y)
        };

        IntPoint::new(I::from_wide(x0 + a0x), I::from_wide(y0 + a0y))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChordCross, SegmentChord};
    use crate::kernel::int::curve::param::SegmentParam;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn crosses_and_resolves_params() {
        let a: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(-10, 0),
            b: IntPoint::new(10, 0),
        };
        let b: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(0, -10),
            b: IntPoint::new(0, 10),
        };
        let Some(ChordCross::Point(point)) = a.cross(&b, 1_i64) else {
            panic!("cross expected")
        };
        assert_eq!(point, IntPoint::ZERO);
        assert_eq!(
            a.param_for_point(point).value(),
            SegmentParam::<i32>::half().value()
        );
        assert_eq!(
            b.param_for_point(point).value(),
            SegmentParam::<i32>::half().value()
        );
    }

    #[test]
    fn detects_overlay() {
        let a: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(0, 0),
            b: IntPoint::new(10, 0),
        };
        let b: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(5, 0),
            b: IntPoint::new(15, 0),
        };
        assert_eq!(a.cross(&b, 1_i64), Some(ChordCross::Overlay));
    }
}
