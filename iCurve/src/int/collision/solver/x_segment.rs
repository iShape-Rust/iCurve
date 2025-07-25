use crate::int::math::ab_segment::IntABSegment;
use crate::int::math::point::IntPoint;
use crate::int::math::triangle::Triangle;
use crate::int::math::x_segment::XSegment;

#[derive(Debug, PartialEq)]
pub enum XResult {
    Segment(XSegment),
    Point(IntPoint),
    None,
}

impl XSegment {
    #[inline]
    pub fn cross(&self, other: &XSegment) -> XResult {
        if !self.is_overlap_xy(other) {
            return XResult::None;
        }

        let a0b0a1 = Triangle::clock_direction(self.a, self.b, other.a);
        let a0b0b1 = Triangle::clock_direction(self.a, self.b, other.b);

        let a1b1a0 = Triangle::clock_direction(other.a, other.b, self.a);
        let a1b1b0 = Triangle::clock_direction(other.a, other.b, self.b);

        let degenerate = a0b0a1 == 0 || a0b0b1 == 0 || a1b1a0 == 0 || a1b1b0 == 0;
        if degenerate {
            return self.degenerate_cross(other);
        }

        if a0b0a1 != a0b0b1 && a1b1a0 != a1b1b0 {
            XResult::Point(self.cross_point(other))
        } else {
            XResult::None
        }
    }

    #[inline]
    fn cross_point(&self, other: &XSegment) -> IntPoint {
        // edges are not parallel

        // Classic approach:

        // let dxA = a0.x - a1.x
        // let dyB = b0.y - b1.y
        // let dyA = a0.y - a1.y
        // let dxB = b0.x - b1.x
        //
        // let xyA = a0.x * a1.y - a0.y * a1.x
        // let xyB = b0.x * b1.y - b0.y * b1.x
        //
        // overflow is possible!
        // let kx = xyA * dxB - dxA * xyB
        //
        // overflow is possible!
        // let ky = xyA * dyB - dyA * xyB
        //
        // let divider = dxA * dyB - dyA * dxB
        //
        // let x = kx / divider
        // let y = ky / divider
        //
        // return FixVec(x, y)

        // offset approach
        // move all picture by -a0. Point a0 will be equal (0, 0)

        // move a0.x to 0
        // move all by a0.x
        let a0x = self.a.x;
        let a0y = self.a.y;

        let a1x = self.b.x - a0x;
        let b0x = other.a.x - a0x;
        let b1x = other.b.x - a0x;

        // move a0.y to 0
        // move all by a0.y
        let a1y = self.b.y - a0y;
        let b0y = other.a.y - a0y;
        let b1y = other.b.y - a0y;

        let dy_b = b0y - b1y;
        let dx_b = b0x - b1x;

        // let xyA = 0
        let xy_b = b0x * b1y - b0y * b1x;

        let x0: i64;
        let y0: i64;

        // a1y and a1x cannot be zero simultaneously, because we will get edge a0<>a1 zero length and it is impossible

        if a1x == 0 {
            // dxB is not zero because it will be parallel case and it's impossible
            x0 = 0;
            y0 = xy_b / dx_b;
        } else if a1y == 0 {
            // dyB is not zero because it will be parallel case and it's impossible
            y0 = 0;
            x0 = -xy_b / dy_b;
        } else {
            // dx_a = -a1x
            // dy_a = -a1y

            let div = (a1y * dx_b - a1x * dy_b) as i128;
            let xy_b128 = xy_b as i128;

            let kx = a1x as i128 * xy_b128;
            let ky = a1y as i128 * xy_b128;

            x0 = (kx / div) as i64;
            y0 = (ky / div) as i64;
        }

        let x = x0 + a0x;
        let y = y0 + a0y;

        IntPoint::new(x, y)
    }

    #[inline]
    fn degenerate_cross(&self, other: &XSegment) -> XResult {
        if self.is_collinear(other) {
            self.degenerate_collinear_cross(other)
        } else {
            self.degenerate_not_collinear_cross(other)
        }
    }

    #[inline]
    fn degenerate_collinear_cross(&self, other: &XSegment) -> XResult {
        if self.a <= other.a {
            self.degenerate_collinear_ordered_cross(other)
        } else {
            other.degenerate_collinear_ordered_cross(self)
        }
    }

    #[inline]
    fn degenerate_collinear_ordered_cross(&self, other: &XSegment) -> XResult {
        if other.b <= self.b {
            XResult::Segment(other.clone())
        } else if other.a < self.b {
            XResult::Segment(XSegment::new(other.a, self.b))
        } else if other.a == self.b {
            XResult::Point(other.a)
        } else {
            XResult::None
        }
    }

    #[inline]
    fn degenerate_not_collinear_cross(&self, other: &XSegment) -> XResult {
        if self.is_on_span(other.a) {
            return XResult::Point(other.a);
        }

        if self.is_on_span(other.b) {
            return XResult::Point(other.b);
        }

        if other.is_on_span(self.a) {
            return XResult::Point(self.a);
        }

        if other.is_on_span(self.b) {
            return XResult::Point(self.b);
        }

        XResult::None
    }
}

#[cfg(test)]
mod tests {
    use crate::int::collision::solver::x_segment::XResult;
    use crate::int::math::point::IntPoint;
    use crate::int::math::x_segment::XSegment;

    #[test]
    fn test_cross_point_0() {
        let s0 = XSegment::new(IntPoint::new(0, -5), IntPoint::new(0, 5));
        let s1 = XSegment::new(IntPoint::new(-5, 0), IntPoint::new(5, 0));

        assert_eq!(s0.cross_point(&s1), IntPoint::new(0, 0));
    }

    #[test]
    fn test_cross_point_1() {
        let s0 = XSegment::new(IntPoint::new(-5, 0), IntPoint::new(5, 0));
        let s1 = XSegment::new(IntPoint::new(-5, -5), IntPoint::new(5, 5));

        assert_eq!(s0.cross_point(&s1), IntPoint::new(0, 0));
    }

    #[test]
    fn test_degenerate_collinear_cross() {
        let cases = [
            [
                IntPoint::new(0, 0),
                IntPoint::new(1, 0),
                IntPoint::new(2, 0),
                IntPoint::new(3, 0),
            ],
            [
                IntPoint::new(0, 0),
                IntPoint::new(0, 1),
                IntPoint::new(0, 2),
                IntPoint::new(0, 3),
            ],
            [
                IntPoint::new(0, 0),
                IntPoint::new(1, 1),
                IntPoint::new(2, 2),
                IntPoint::new(3, 3),
            ],
        ];

        for [a, b, c, d] in cases {
            let ab = XSegment::new(a, b);
            let ac = XSegment::new(a, c);
            let ad = XSegment::new(a, d);

            let bc = XSegment::new(b, c);
            let bd = XSegment::new(b, d);

            let cd = XSegment::new(c, d);

            // ab

            assert_eq!(ab.degenerate_collinear_cross(&ab), XResult::Segment(ab));
            assert_eq!(ab.degenerate_collinear_cross(&ac), XResult::Segment(ab));
            assert_eq!(ab.degenerate_collinear_cross(&ad), XResult::Segment(ab));

            assert_eq!(ab.degenerate_collinear_cross(&bc), XResult::Point(b));
            assert_eq!(ab.degenerate_collinear_cross(&bd), XResult::Point(b));

            assert_eq!(ab.degenerate_collinear_cross(&cd), XResult::None);

            // ac

            assert_eq!(ac.degenerate_collinear_cross(&ab), XResult::Segment(ab));
            assert_eq!(ac.degenerate_collinear_cross(&ac), XResult::Segment(ac));
            assert_eq!(ac.degenerate_collinear_cross(&ad), XResult::Segment(ac));

            assert_eq!(ac.degenerate_collinear_cross(&bc), XResult::Segment(bc));
            assert_eq!(ac.degenerate_collinear_cross(&bd), XResult::Segment(bc));

            assert_eq!(ac.degenerate_collinear_cross(&cd), XResult::Point(c));

            // ad

            assert_eq!(ad.degenerate_collinear_cross(&ab), XResult::Segment(ab));
            assert_eq!(ad.degenerate_collinear_cross(&ac), XResult::Segment(ac));
            assert_eq!(ad.degenerate_collinear_cross(&ad), XResult::Segment(ad));

            assert_eq!(ad.degenerate_collinear_cross(&bc), XResult::Segment(bc));
            assert_eq!(ad.degenerate_collinear_cross(&bd), XResult::Segment(bd));

            assert_eq!(ad.degenerate_collinear_cross(&cd), XResult::Segment(cd));

            // bc

            assert_eq!(bc.degenerate_collinear_cross(&ab), XResult::Point(b));
            assert_eq!(bc.degenerate_collinear_cross(&ac), XResult::Segment(bc));
            assert_eq!(bc.degenerate_collinear_cross(&ad), XResult::Segment(bc));

            assert_eq!(bc.degenerate_collinear_cross(&bc), XResult::Segment(bc));
            assert_eq!(bc.degenerate_collinear_cross(&bd), XResult::Segment(bc));

            assert_eq!(bc.degenerate_collinear_cross(&cd), XResult::Point(c));

            // bd

            assert_eq!(bd.degenerate_collinear_cross(&ab), XResult::Point(b));
            assert_eq!(bd.degenerate_collinear_cross(&ac), XResult::Segment(bc));
            assert_eq!(bd.degenerate_collinear_cross(&ad), XResult::Segment(bd));

            assert_eq!(bd.degenerate_collinear_cross(&bc), XResult::Segment(bc));
            assert_eq!(bd.degenerate_collinear_cross(&bd), XResult::Segment(bd));

            assert_eq!(bd.degenerate_collinear_cross(&cd), XResult::Segment(cd));

            // cd

            assert_eq!(cd.degenerate_collinear_cross(&ab), XResult::None);
            assert_eq!(cd.degenerate_collinear_cross(&ac), XResult::Point(c));
            assert_eq!(cd.degenerate_collinear_cross(&ad), XResult::Segment(cd));

            assert_eq!(cd.degenerate_collinear_cross(&bc), XResult::Point(c));
            assert_eq!(cd.degenerate_collinear_cross(&bd), XResult::Segment(cd));

            assert_eq!(cd.degenerate_collinear_cross(&cd), XResult::Segment(cd));
        }
    }
}
