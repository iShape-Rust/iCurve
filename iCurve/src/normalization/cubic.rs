use crate::collections::stack_vec::StackVec;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::point_at::PointAt;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use crate::kernel::math::quadratic_equation::QuadraticEquation;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

struct CubicSelfIntersection<T: FloatNumber> {
    t0: T,
    t1: T,
    point: FloatPoint<T>,
}

impl<T: FloatNumber> CubicSegment<T> {
    #[inline]
    pub(super) fn try_with_adapter<I: IntNumber>(
        self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<StackVec<Segment<T>, 4>, FloatPointAdapterRangeError> {
        let [p0, p1, p2, p3] = self.control_points;

        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;

        let mut segments = StackVec::new();

        if q0 == q3 {
            // A closed cubic with two distinct control directions can enclose area;
            // otherwise it is only a normalized spike.
            if q0 != q1 && q0 != q2 && q1 != q2 {
                let [first, last] = self.split_at(T::HALF);

                segments.push_some(first.try_cubic_without_self_intersection(adapter)?);
                segments.push_some(last.try_cubic_without_self_intersection(adapter)?);
            }
            return Ok(segments);
        }

        if q1 == q2 {
            // Equal middle controls reduce the cubic to a quadratic.
            segments.push_some(
                QuadSegment {
                    control_points: [p0, p1, p3],
                }
                .try_with_adapter(adapter)?,
            );
            return Ok(segments);
        }

        if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            // All controls lie on the chord, so the cubic contributes a line.
            segments.push_some(
                LineSegment {
                    control_points: [p0, p3],
                }
                .try_with_adapter(adapter)?,
            );
            return Ok(segments);
        }

        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };

        let Some(intersection) = cubic.resolve_self_intersection() else {
            segments.push_some(cubic.try_cubic_without_self_intersection(adapter)?);
            return Ok(segments);
        };

        let (t0, t1) = if intersection.t0 < intersection.t1 {
            (intersection.t0, intersection.t1)
        } else {
            (intersection.t1, intersection.t0)
        };

        let [mut first, rest] = self.split_at(t0);
        let t = (t1 - t0) / (T::ONE - t0);
        let [mut middle, mut last] = rest.split_at(t);

        let point = intersection.point;
        first.control_points[3] = point;
        middle.control_points[0] = point;
        middle.control_points[3] = point;
        last.control_points[0] = point;

        let [middle_0, middle_1] = middle.split_at(T::HALF);

        // After splitting at the loop crossing, every part is loop-free.
        segments.push_some(first.try_cubic_without_self_intersection(adapter)?);
        segments.push_some(middle_0.try_cubic_without_self_intersection(adapter)?);
        segments.push_some(middle_1.try_cubic_without_self_intersection(adapter)?);
        segments.push_some(last.try_cubic_without_self_intersection(adapter)?);

        Ok(segments)
    }

    fn resolve_self_intersection(&self) -> Option<CubicSelfIntersection<T>> {
        let [p0, p1, p2, p3] = self.control_points;

        // Cubic Bezier in Bernstein form:
        //
        //   P(t) = (1 - t)^3 * P0
        //        + 3 * (1 - t)^2 * t * P1
        //        + 3 * (1 - t) * t^2 * P2
        //        + t^3 * P3
        //
        // Expanding it to power form gives:
        //
        //   P(t) = a * t^3 + b * t^2 + c * t + P0
        //
        // A self-intersection has two distinct parameters u and v where:
        //
        //   P(u) = P(v)
        //
        // After factoring P(u) - P(v) by (u - v), and denoting:
        //
        //   s = u + v
        //   p = u * v
        //
        // we get the vector equation:
        //
        //   a * (s^2 - p) + b * s + c = 0
        //
        // Taking the cross product with a removes the a term:
        //
        //   s = -cross(a, c) / cross(a, b)
        //
        // Then p is the projection of q onto a, where:
        //
        //   q = a * s^2 + b * s + c = a * p
        //
        // Finally u and v are the roots of:
        //
        //   t^2 - s * t + p = 0
        let a = p3 - p2 * T::THREE + p1 * T::THREE - p0;
        let b = (p2 - p1 * T::TWO + p0) * T::THREE;
        let c = (p1 - p0) * T::THREE;

        let ab = a.cross_product(b);
        if ab == T::ZERO {
            return None;
        }

        let aa = a.sqr_length();
        if aa == T::ZERO {
            return None;
        }

        let s = -a.cross_product(c) / ab;
        let q = a * (s * s) + b * s + c;
        let p = a.dot_product(q) / aa;

        let [t0, t1] = QuadraticEquation::solve(T::ONE, -s, p)?;
        if !(T::ZERO < t0 && t0 < T::ONE && T::ZERO < t1 && t1 < T::ONE && t0 != t1) {
            return None;
        }

        Some(CubicSelfIntersection {
            t0,
            t1,
            point: self.point_at(SegmentParam::Inner(t0)),
        })
    }
    fn try_cubic_without_self_intersection<I: IntNumber>(
        self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<Segment<T>>, FloatPointAdapterRangeError> {
        let [p0, p1, p2, p3] = self.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;

        // Loop-free closed sub-cubic has no overlay contribution.
        if q0 == q3 {
            Ok(None)
        } else if q1 == q2 {
            // Equal middle controls reduce the cubic to a quadratic.
            QuadSegment {
                control_points: [p0, p1, p3],
            }
            .try_with_adapter(adapter)
        } else if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            // All controls lie on the chord, so the cubic contributes a line.
            LineSegment {
                control_points: [p0, p3],
            }
            .try_with_adapter(adapter)
        } else {
            Ok(Some(Segment::Cubic(self)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalization::test_utils::{assert_control_points_eq, assert_point_eq};

    #[test]
    fn drops_closed_cubic_spike() {
        let cubic = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [1.0, 0.0].into(),
                [1.0, 0.0].into(),
                [0.0, 0.0].into(),
            ],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        assert!(segments.is_empty());
    }

    #[test]
    fn splits_closed_cubic_with_area() {
        let cubic = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [1.0, 2.0].into(),
                [-1.0, 2.0].into(),
                [0.0, 0.0].into(),
            ],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        assert_eq!(segments.as_slice().len(), 2);
        match (&segments.as_slice()[0], &segments.as_slice()[1]) {
            (Segment::Cubic(first), Segment::Cubic(last)) => {
                assert_point_eq(first.control_points[0], [0.0, 0.0]);
                assert_point_eq(first.control_points[3], [0.0, 1.5]);
                assert_point_eq(last.control_points[0], [0.0, 1.5]);
                assert_point_eq(last.control_points[3], [0.0, 0.0]);
            }
            _ => panic!("expected cubic segments"),
        }
    }

    #[test]
    fn reduces_cubic_with_equal_middle_controls_to_quad() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 1.0);
        let p3 = FloatPoint::new(2.0, 0.0);
        let cubic = CubicSegment {
            control_points: [p0, p1, p1, p3],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        match segments.as_slice() {
            [Segment::Quad(segment)] => assert_control_points_eq(segment.control_points, [p0, p1, p3]),
            _ => panic!("expected one quad segment"),
        }
    }

    #[test]
    fn reduces_collinear_cubic_to_line() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 0.0);
        let p2 = FloatPoint::new(2.0, 0.0);
        let p3 = FloatPoint::new(3.0, 0.0);
        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        match segments.as_slice() {
            [Segment::Line(segment)] => assert_control_points_eq(segment.control_points, [p0, p3]),
            _ => panic!("expected one line segment"),
        }
    }

    #[test]
    fn keeps_non_intersecting_cubic() {
        let cubic = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        match segments.as_slice() {
            [Segment::Cubic(segment)] => {
                assert_control_points_eq(segment.control_points, cubic.control_points)
            }
            _ => panic!("expected one cubic segment"),
        }
    }

    #[test]
    fn splits_self_intersecting_cubic() {
        let cubic = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [-3.0, -3.0].into(),
                [-3.0, -2.0].into(),
                [-2.0, -2.0].into(),
            ],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segments = cubic.try_with_adapter(&adapter).unwrap();

        assert_eq!(segments.as_slice().len(), 4);
        let point = [-2.3615160349854225, -2.0466472303206995];
        match segments.as_slice() {
            [
                Segment::Cubic(first),
                Segment::Cubic(middle_0),
                Segment::Cubic(middle_1),
                Segment::Cubic(last),
            ] => {
                assert_point_eq(first.control_points[3], point);
                assert_point_eq(middle_0.control_points[0], point);
                assert_point_eq(middle_1.control_points[3], point);
                assert_point_eq(last.control_points[0], point);
            }
            _ => panic!("expected cubic segments"),
        }
    }

    #[test]
    fn drops_loop_free_closed_sub_cubic() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 0.0);
        let cubic = CubicSegment {
            control_points: [p0, p1, p1, p0],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(cubic.control_points.iter());

        let segment = cubic.try_cubic_without_self_intersection(&adapter).unwrap();

        assert!(segment.is_none());
    }

    #[test]
    fn finds_cubic_self_intersection() {
        let intersection = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [-3.0, -3.0].into(),
                [-3.0, -2.0].into(),
                [-2.0, -2.0].into(),
            ],
        }
        .resolve_self_intersection()
        .unwrap();

        assert!((intersection.t0.to_f64() - 3.0 / 7.0).abs() < 0.000001);
        assert!((intersection.t1.to_f64() - 6.0 / 7.0).abs() < 0.000001);
        assert!((intersection.point.x.to_f64() + 2.3615160349854225).abs() < 0.000001);
        assert!((intersection.point.y.to_f64() + 2.0466472303206995).abs() < 0.000001);
    }

    #[test]
    fn ignores_non_intersecting_cubic() {
        let intersection = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        }
        .resolve_self_intersection();

        assert!(intersection.is_none());
    }
}
