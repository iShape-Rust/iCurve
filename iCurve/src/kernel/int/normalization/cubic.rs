use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::curve::split_at::SplitAt;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::IntPoint;

struct CubicSelfIntersection<I: IntNumber> {
    t0: SegmentParam<I>,
    t1: SegmentParam<I>,
    point: IntPoint<I>,
}

impl<I: IntNumber> CubicSegment<I> {
    #[inline]
    pub(crate) fn try_segment(self) -> StackVec<Segment<I>, 4> {
        let [p0, p1, p2, p3] = self.control_points;
        let mut segments = StackVec::new();

        if p0 == p3 {
            // A closed cubic with two distinct control directions can enclose area;
            // otherwise it is only a normalized spike.
            if p0 != p1 && p0 != p2 && p1 != p2 {
                let [first, last] = self.split_at(SegmentParam::half());

                segments.push_some(first.try_cubic_without_self_intersection());
                segments.push_some(last.try_cubic_without_self_intersection());
            }
            return segments;
        }

        if p1 == p2 {
            // Equal middle controls reduce the cubic to a quadratic.
            segments.push_some(
                QuadSegment {
                    control_points: [p0, p1, p3],
                }
                .try_segment(),
            );
            return segments;
        }

        if Triangle::is_line(p0, p1, p3) && Triangle::is_line(p0, p2, p3) {
            // All controls lie on the chord, so the cubic contributes a line.
            segments.push_some(
                LineSegment {
                    control_points: [p0, p3],
                }
                .try_segment(),
            );
            return segments;
        }

        let Some(intersection) = self.resolve_self_intersection() else {
            segments.push_some(self.try_cubic_without_self_intersection());
            return segments;
        };

        let (t0, t1) = if intersection.t0.value() < intersection.t1.value() {
            (intersection.t0, intersection.t1)
        } else {
            (intersection.t1, intersection.t0)
        };

        let [mut first, rest] = self.split_at(t0);
        let t = local_segment_param(t0, t1);
        let [mut middle, mut last] = rest.split_at(t);

        let point = intersection.point;
        first.control_points[3] = point;
        middle.control_points[0] = point;
        middle.control_points[3] = point;
        last.control_points[0] = point;

        let [middle_0, middle_1] = middle.split_at(SegmentParam::half());

        // After splitting at the loop crossing, every part is loop-free.
        segments.push_some(first.try_cubic_without_self_intersection());
        segments.push_some(middle_0.try_cubic_without_self_intersection());
        segments.push_some(middle_1.try_cubic_without_self_intersection());
        segments.push_some(last.try_cubic_without_self_intersection());

        segments
    }

    fn resolve_self_intersection(&self) -> Option<CubicSelfIntersection<I>> {
        let [p0, p1, p2, p3] = self.control_points;
        let three = I::Wide::from_u32(3);

        // Cubic Bezier in Bernstein form:
        //
        //   P(t) = (1 - t)^3 * P0
        //        + 3 * (1 - t)^2 * t * P1
        //        + 3 * (1 - t) * t^2 * P2
        //        + t^3 * P3
        //
        // In power form:
        //
        //   P(t) = a * t^3 + b * t^2 + c * t + P0
        //
        // where:
        //
        //   a = -P0 + 3P1 - 3P2 + P3
        //   b =  3P0 - 6P1 + 3P2
        //   c = -3P0 + 3P1
        //
        // A self-intersection has two distinct parameters u and v:
        //
        //   P(u) = P(v), u != v
        //
        // Subtracting:
        //
        //   P(u) - P(v) = a * (u^3 - v^3) + b * (u^2 - v^2) + c * (u - v)
        //
        // Factor by (u - v):
        //
        //   P(u) - P(v) = (u - v) * [a * (u^2 + uv + v^2) + b * (u + v) + c]
        //
        // Since u != v, the bracketed term must be zero:
        //
        //   a * (u^2 + uv + v^2) + b * (u + v) + c = 0
        //
        // Introduce the symmetric values:
        //
        //   s = u + v
        //   p = u * v
        //
        // Since:
        //
        //   u^2 + uv + v^2 = (u + v)^2 - uv = s^2 - p
        //
        // we get:
        //
        //   a * (s^2 - p) + b * s + c = 0
        //
        // Rearranged:
        //
        //   a * s^2 + b * s + c = a * p
        //
        // Let:
        //
        //   q = a * s^2 + b * s + c
        //
        // Then:
        //
        //   q = a * p
        //
        // To find s, take the cross product with a:
        //
        //   cross(a, a * (s^2 - p) + b * s + c) = 0
        //
        // The first term disappears because cross(a, a * scalar) == 0:
        //
        //   s * cross(a, b) + cross(a, c) = 0
        //
        // Therefore:
        //
        //   s = -cross(a, c) / cross(a, b)
        //
        // This is valid only for the non-degenerate case:
        //
        //   cross(a, b) != 0
        //
        // Since u and v must be inside the Bezier parameter range [0, 1],
        // their sum must satisfy:
        //
        //   0 < s < 2
        //
        // This gives an early rejection before computing p.
        //
        // After s is known, compute:
        //
        //   q = a * s^2 + b * s + c
        //
        // Since q = a * p, p is the scalar projection of q onto a:
        //
        //   p = dot(q, a) / dot(a, a)
        //
        // Also q must be parallel to a:
        //
        //   cross(q, a) == 0
        //
        // Now u and v are the two roots of:
        //
        //   t^2 - s * t + p = 0
        //
        // because:
        //
        //   (t - u)(t - v) = t^2 - (u + v)t + uv
        //
        // The discriminant is:
        //
        //   D = s^2 - 4p
        //
        // For two distinct parameters:
        //
        //   D > 0
        //
        // Then:
        //
        //   u = (s - sqrt(D)) / 2
        //   v = (s + sqrt(D)) / 2
        //
        // Final validity checks:
        //
        //   0 <= u <= 1
        //   0 <= v <= 1
        //   u != v
        //   P(u) == P(v)
        //
        // For an internal self-intersection, excluding endpoint-only cases:
        //
        //   0 < u < 1
        //   0 < v < 1

        let a = IntVector::<I>::new(
            p3.x.to_wide() - three * p2.x.to_wide() + three * p1.x.to_wide() - p0.x.to_wide(),
            p3.y.to_wide() - three * p2.y.to_wide() + three * p1.y.to_wide() - p0.y.to_wide(),
        );
        let b = IntVector::<I>::new(
            three * (p2.x.to_wide() - I::Wide::TWO * p1.x.to_wide() + p0.x.to_wide()),
            three * (p2.y.to_wide() - I::Wide::TWO * p1.y.to_wide() + p0.y.to_wide()),
        );
        let c = IntVector::<I>::new(
            three * (p1.x.to_wide() - p0.x.to_wide()),
            three * (p1.y.to_wide() - p0.y.to_wide()),
        );

        let ab = a.cross_product(b);
        if ab == I::Wide::ZERO {
            return None;
        }

        let aa = a.sqr_length();
        if aa == I::Wide::ZERO {
            return None;
        }

        let ac = a.cross_product(c);

        // s = -a*c / ab
        let s_scaled = FixedScale::<I>::div_to_scaled_round(-ac, ab);

        // 0 < s < 2
        if s_scaled <= I::Wide::ZERO || s_scaled >= I::TWO.to_scaled_wide() {
            return None;
        }

        // ss = s^2 where s > 0
        let ss_scaled = (s_scaled * s_scaled).shr_round_positive(FixedScale::<I>::SHIFT);

        let q_scaled = IntVector::<I>::new(
            a.x * ss_scaled + b.x * s_scaled + c.x.to_scaled(),
            a.y * ss_scaled + b.y * s_scaled + c.y.to_scaled(),
        );

        let p_scaled = FixedScale::<I>::div_round(a.dot_product(q_scaled), aa);

        // 0 < p < 1
        if p_scaled <= I::Wide::ZERO || p_scaled >= I::ONE.to_scaled_wide() {
            return None;
        }

        let [t0, t1] = solve_unit_quadratic::<I>(ss_scaled, s_scaled, p_scaled)?;

        Some(CubicSelfIntersection {
            t0,
            t1,
            point: self.control_points.point_at(t0),
        })
    }

    fn try_cubic_without_self_intersection(self) -> Option<Segment<I>> {
        let [p0, p1, p2, p3] = self.control_points;

        // Loop-free closed sub-cubic has no overlay contribution.
        if p0 == p3 {
            None
        } else if p1 == p2 {
            // Equal middle controls reduce the cubic to a quadratic.
            QuadSegment {
                control_points: [p0, p1, p3],
            }
            .try_segment()
        } else if Triangle::is_line(p0, p1, p3) && Triangle::is_line(p0, p2, p3) {
            // All controls lie on the chord, so the cubic contributes a line.
            LineSegment {
                control_points: [p0, p3],
            }
            .try_segment()
        } else {
            Some(Segment::Cubic(self))
        }
    }
}

fn local_segment_param<I: IntNumber>(t0: SegmentParam<I>, t1: SegmentParam<I>) -> SegmentParam<I> {
    let numerator = t1.value() - t0.value();
    let denominator = SegmentParam::<I>::DENOMINATOR - t0.value();

    SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator))
}

fn solve_unit_quadratic<I: IntNumber>(
    ss_scaled: I::Wide,
    s_scaled: I::Wide,
    p_scaled: I::Wide,
) -> Option<[SegmentParam<I>; 2]> {
    let d_scaled = ss_scaled - I::Wide::FOUR * p_scaled;
    if d_scaled <= I::Wide::ZERO {
        // skip equal roots
        return None;
    }

    // Safe because 0 < s_scaled < 2D, 0 < p_scaled < D,
    // d_scaled = ss_scaled - 4p_scaled, and d_scaled > 0.
    let sqrt_d_scaled = d_scaled.to_scaled().isqrt();

    let t0 = FixedScale::<I>::div_round(s_scaled - sqrt_d_scaled, I::Wide::TWO);
    let t1 = FixedScale::<I>::div_round(s_scaled + sqrt_d_scaled, I::Wide::TWO);

    debug_assert!(t0 < t1);
    if t0 <= I::Wide::ZERO || t1 >= SegmentParam::<I>::DENOMINATOR {
        return None;
    }

    Some([
        SegmentParam::new(I::from_wide(t0)),
        SegmentParam::new(I::from_wide(t1)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_closed_cubic_spike() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(1, 0),
                IntPoint::new(1, 0),
                IntPoint::new(0, 0),
            ],
        };

        let segments = cubic.try_segment();

        assert!(segments.is_empty());
    }

    #[test]
    fn splits_closed_cubic_with_area() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(2, 4),
                IntPoint::new(-2, 4),
                IntPoint::new(0, 0),
            ],
        };

        let segments = cubic.try_segment();

        assert_eq!(segments.as_slice().len(), 2);
        match segments.as_slice() {
            [Segment::Cubic(first), Segment::Cubic(last)] => {
                assert_eq!(first.control_points[0], IntPoint::new(0, 0));
                assert_eq!(first.control_points[3], IntPoint::new(-1, 3));
                assert_eq!(last.control_points[0], IntPoint::new(-1, 3));
                assert_eq!(last.control_points[3], IntPoint::new(0, 0));
            }
            _ => panic!("expected cubic segments"),
        }
    }

    #[test]
    fn reduces_cubic_with_equal_middle_controls_to_quad() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 1);
        let p3 = IntPoint::new(2, 0);
        let cubic = CubicSegment {
            control_points: [p0, p1, p1, p3],
        };

        let segments = cubic.try_segment();

        match segments.as_slice() {
            [Segment::Quad(segment)] => assert_eq!(segment.control_points, [p0, p1, p3]),
            _ => panic!("expected one quad segment"),
        }
    }

    #[test]
    fn reduces_collinear_cubic_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 0);
        let p2 = IntPoint::new(2, 0);
        let p3 = IntPoint::new(3, 0);
        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };

        let segments = cubic.try_segment();

        match segments.as_slice() {
            [Segment::Line(segment)] => assert_eq!(segment.control_points, [p0, p3]),
            _ => panic!("expected one line segment"),
        }
    }

    #[test]
    fn keeps_non_intersecting_cubic() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(1, 2),
                IntPoint::new(3, 2),
                IntPoint::new(4, 0),
            ],
        };

        let segments = cubic.try_segment();

        match segments.as_slice() {
            [Segment::Cubic(segment)] => assert_eq!(segment.control_points, cubic.control_points),
            _ => panic!("expected one cubic segment"),
        }
    }

    #[test]
    fn finds_cubic_self_intersection_with_scaled_divisions() {
        let intersection = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(-21, -21),
                IntPoint::new(-21, -14),
                IntPoint::new(-14, -14),
            ],
        }
        .resolve_self_intersection()
        .unwrap();

        assert!((intersection.t0.value() - SegmentParam::<i32>::from_int(3, 7).value()).abs() <= 1);
        assert!((intersection.t1.value() - SegmentParam::<i32>::from_int(6, 7).value()).abs() <= 1);
        assert_eq!(intersection.point, IntPoint::new(-17, -14));
    }
}
