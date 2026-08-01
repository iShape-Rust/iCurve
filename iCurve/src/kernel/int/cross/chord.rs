use crate::int::CurveInt;
use crate::kernel::int::curve::chord::SegmentChord;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChordCross<I: CurveInt> {
    Point(IntPoint<I>),
    Overlay,
}

impl<I: CurveInt> SegmentChord<I> {
    pub(crate) fn cross(&self, other: &Self, radius: I::Wide) -> Option<ChordCross<I>> {
        let va = self.vector();
        let vb = other.vector();

        let a0 = va.cross_product(other.a - self.a);
        let a1 = va.cross_product(other.b - self.a);
        let b0 = vb.cross_product(self.a - other.a);
        let b1 = vb.cross_product(self.b - other.a);
        let zero = I::Wide::ZERO;

        if a0 != zero && a1 != zero && b0 != zero && b1 != zero {
            let is_cross = (a0 < zero) != (a1 < zero) && (b0 < zero) != (b1 < zero);
            return is_cross.then(|| ChordCross::Point(self.middle_cross_point(other, radius)));
        }

        if a0 == zero && a1 == zero && b0 == zero && b1 == zero {
            return self.collinear_cross(other);
        }

        if a0 == zero && self.contains_collinear(other.a) {
            return Some(ChordCross::Point(other.a));
        }
        if a1 == zero && self.contains_collinear(other.b) {
            return Some(ChordCross::Point(other.b));
        }
        if b0 == zero && other.contains_collinear(self.a) {
            return Some(ChordCross::Point(self.a));
        }
        if b1 == zero && other.contains_collinear(self.b) {
            return Some(ChordCross::Point(self.b));
        }

        None
    }

    fn collinear_cross(&self, other: &Self) -> Option<ChordCross<I>> {
        let candidates = [
            (self.a, other.contains_collinear(self.a)),
            (self.b, other.contains_collinear(self.b)),
            (other.a, self.contains_collinear(other.a)),
            (other.b, self.contains_collinear(other.b)),
        ];
        let mut point = None;

        for (candidate, is_common) in candidates {
            if !is_common {
                continue;
            }
            if point.is_some_and(|point| point != candidate) {
                return Some(ChordCross::Overlay);
            }
            point = Some(candidate);
        }

        point.map(ChordCross::Point)
    }

    #[inline]
    fn contains_collinear(&self, point: IntPoint<I>) -> bool {
        self.a.x.min(self.b.x) <= point.x
            && point.x <= self.a.x.max(self.b.x)
            && self.a.y.min(self.b.y) <= point.y
            && point.y <= self.a.y.max(self.b.y)
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

    #[test]
    fn detects_shared_endpoint() {
        let a: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(-10, -10),
            b: IntPoint::ZERO,
        };
        let b: SegmentChord<i32> = SegmentChord {
            a: IntPoint::ZERO,
            b: IntPoint::new(10, -10),
        };

        assert_eq!(a.cross(&b, 1_i64), Some(ChordCross::Point(IntPoint::ZERO)));
    }

    #[test]
    fn detects_collinear_endpoint() {
        let a: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(0, 0),
            b: IntPoint::new(10, 0),
        };
        let b: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(10, 0),
            b: IntPoint::new(20, 0),
        };

        assert_eq!(a.cross(&b, 1_i64), Some(ChordCross::Point(IntPoint::new(10, 0))));
    }

    #[test]
    fn rejects_disjoint_collinear_chords() {
        let a: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(0, 0),
            b: IntPoint::new(10, 0),
        };
        let b: SegmentChord<i32> = SegmentChord {
            a: IntPoint::new(20, 0),
            b: IntPoint::new(30, 0),
        };

        assert_eq!(a.cross(&b, 1_i64), None);
    }
}
