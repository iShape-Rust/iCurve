use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::curve::split_at::SplitAt;
use crate::kernel::int::normalization::monotone::decomposition::roots_to_segments;
use crate::kernel::int::normalization::unit_quadratic::solve_unit_quadratic;
use core::cmp::Ordering;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::signed_product::SignedProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_float::triangle::Triangle;
use i_overlay::i_shape::int::IntPoint;

struct CubicSelfIntersection<I: IntNumber> {
    t0: SegmentParam<I>,
    t1: SegmentParam<I>,
    point: IntPoint<I>,
}

pub(crate) enum CubicSShapeNormalization<I: IntNumber> {
    NoS(CubicSegment<I>),
    Pieces([CubicSegment<I>; 2]),
}

impl<I: IntNumber> CubicSegment<I> {
    pub(crate) fn split_at_cusps(&self) -> StackVec<Self, 3> {
        roots_to_segments(self, self.cusp_roots())
    }

    fn cusp_roots(&self) -> StackVec<SegmentParam<I>, 2> {
        let [p0, p1, p2, p3] = self.control_points;
        let u = IntVector::<I>::new(p1.x.to_wide() - p0.x.to_wide(), p1.y.to_wide() - p0.y.to_wide());
        let v = IntVector::<I>::new(p2.x.to_wide() - p1.x.to_wide(), p2.y.to_wide() - p1.y.to_wide());
        let w = IntVector::<I>::new(p3.x.to_wide() - p2.x.to_wide(), p3.y.to_wide() - p2.y.to_wide());
        let s = IntVector::<I>::new(v.x - u.x, v.y - u.y);
        let r = IntVector::<I>::new(w.x - I::Wide::TWO * v.x + u.x, w.y - I::Wide::TWO * v.y + u.y);

        let x_is_zero = r.x == I::Wide::ZERO && s.x == I::Wide::ZERO && u.x == I::Wide::ZERO;
        let y_is_zero = r.y == I::Wide::ZERO && s.y == I::Wide::ZERO && u.y == I::Wide::ZERO;
        let x_roots = solve_unit_quadratic::<I>(r.x, I::Wide::TWO * s.x, u.x);
        let y_roots = solve_unit_quadratic::<I>(r.y, I::Wide::TWO * s.y, u.y);

        if x_is_zero {
            return y_roots;
        }
        if y_is_zero {
            return x_roots;
        }

        let mut roots = StackVec::new();
        for &x in x_roots.as_slice() {
            for &y in y_roots.as_slice() {
                let delta = x.value() - y.value();
                let distance = if delta < I::Wide::ZERO { -delta } else { delta };
                if distance <= I::Wide::ONE {
                    roots.push(x);
                    break;
                }
            }
        }
        roots.as_mut_slice().sort_unstable_by_key(|root| root.value());
        roots.dedup();
        roots
    }

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

    #[inline]
    pub(crate) fn normalize_monotone_without_s_shape(self) -> CubicSShapeNormalization<I> {
        match self.s_shape_split_param() {
            Some(t) => CubicSShapeNormalization::Pieces(self.split_at(t)),
            None => CubicSShapeNormalization::NoS(self),
        }
    }

    pub(crate) fn s_shape_split_param(&self) -> Option<SegmentParam<I>> {
        let [p0, p1, p2, p3] = self.control_points;

        // For P'(t) = 3 * (u + 2s*t + r*t^2) and P''(t) = 6 * (s + r*t),
        // inflections satisfy cross(P'(t), P''(t)) = 0:
        //
        //   cross(s, r) * t^2 + cross(u, r) * t + cross(u, s) = 0
        //
        // where:
        //
        //   u = P1 - P0
        //   s = (P2 - P1) - u
        //   r = (P3 - P2) - 2(P2 - P1) + u
        let u = IntVector::<I>::new(p1.x.to_wide() - p0.x.to_wide(), p1.y.to_wide() - p0.y.to_wide());
        let v = IntVector::<I>::new(p2.x.to_wide() - p1.x.to_wide(), p2.y.to_wide() - p1.y.to_wide());
        let w = IntVector::<I>::new(p3.x.to_wide() - p2.x.to_wide(), p3.y.to_wide() - p2.y.to_wide());

        let s = IntVector::<I>::new(v.x - u.x, v.y - u.y);
        let r = IntVector::<I>::new(w.x - I::Wide::TWO * v.x + u.x, w.y - I::Wide::TWO * v.y + u.y);

        let a = s.cross_product(r);
        let b = u.cross_product(r);
        let c = u.cross_product(s);

        if a == I::Wide::ZERO {
            return unit_signed_ratio::<I>(c.unsigned_abs(), c > I::Wide::ZERO, b);
        }

        // An endpoint inflection is not a split, but the remaining linear
        // factor may still have one internal root.
        if c == I::Wide::ZERO {
            return unit_signed_ratio::<I>(b.unsigned_abs(), b > I::Wide::ZERO, a);
        }

        let start_value = s_shape_value::<I>(a, b, c, I::Wide::ZERO)?;
        let end_value = s_shape_value::<I>(a, b, c, SegmentParam::<I>::DENOMINATOR)?;
        if end_value.sign() == Ordering::Equal {
            return unit_signed_ratio::<I>(c.unsigned_abs(), c < I::Wide::ZERO, a);
        }

        // A monotone non-self-intersecting cubic has at most one simple
        // internal inflection. Such a root changes the curvature sign, while
        // a double root only touches zero and must not create an S-shape split.
        if start_value.is_negative() == end_value.is_negative() {
            return None;
        }

        find_s_shape_root::<I>(a, b, c, start_value, end_value)
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

        let p_scaled = unit_dot_ratio_scaled(a, q_scaled, aa)?;

        // 0 < p < 1
        if p_scaled <= I::Wide::ZERO || p_scaled >= I::ONE.to_scaled_wide() {
            return None;
        }

        let [t0, t1] = solve_self_intersection_params::<I>(ss_scaled, s_scaled, p_scaled)?;

        Some(CubicSelfIntersection {
            t0,
            t1,
            point: self.control_points.point_at(t0),
        })
    }

    pub(crate) fn try_cubic_without_self_intersection(self) -> Option<Segment<I>> {
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

/// Finds the unique simple S-shape root on the fixed-point unit interval.
///
/// The caller guarantees opposite non-zero curvature signs at the interval
/// ends. Binary search preserves that bracket until the two closest fixed-point
/// parameters around the root remain.
fn find_s_shape_root<I: IntNumber>(
    a: I::Wide,
    b: I::Wide,
    c: I::Wide,
    mut lo_value: SignedProduct<I::Wide>,
    mut hi_value: SignedProduct<I::Wide>,
) -> Option<SegmentParam<I>> {
    let mut lo = I::Wide::ZERO;
    let mut hi = SegmentParam::<I>::DENOMINATOR;

    while hi - lo > I::Wide::ONE {
        let mid = (lo + hi) >> 1;
        let mid_value = s_shape_value::<I>(a, b, c, mid)?;

        match mid_value.sign() {
            Ordering::Equal => return Some(SegmentParam::new(I::from_wide(mid))),
            _ if mid_value.is_negative() == lo_value.is_negative() => {
                lo = mid;
                lo_value = mid_value;
            }
            _ => {
                hi = mid;
                hi_value = mid_value;
            }
        }
    }

    let value = if lo_value.magnitude() <= hi_value.magnitude() {
        lo
    } else {
        hi
    };
    if value <= I::Wide::ZERO || value >= SegmentParam::<I>::DENOMINATOR {
        None
    } else {
        Some(SegmentParam::new(I::from_wide(value)))
    }
}

/// Evaluates `F^2 * (a*t^2 + b*t + c)` for `t = scaled_t / F`.
/// Every signed term remains in the double-width unsigned representation.
fn s_shape_value<I: IntNumber>(
    a: I::Wide,
    b: I::Wide,
    c: I::Wide,
    scaled_t: I::Wide,
) -> Option<SignedProduct<I::Wide>> {
    debug_assert!(scaled_t >= I::Wide::ZERO);
    debug_assert!(scaled_t <= SegmentParam::<I>::DENOMINATOR);

    let scale = SegmentParam::<I>::DENOMINATOR;
    let tt = scaled_t * scaled_t;
    let tf = scaled_t * scale;
    let ff = scale * scale;

    let at = SignedProduct::multiply(a, tt);
    let bt = SignedProduct::multiply(b, tf);
    let ct = SignedProduct::multiply(c, ff);

    at.checked_add(bt)?.checked_add(ct)
}

/// Returns a strictly internal fixed-point ratio from signed magnitude parts.
fn unit_signed_ratio<I: IntNumber>(
    numerator: I::WideUInt,
    numerator_negative: bool,
    denominator: I::Wide,
) -> Option<SegmentParam<I>> {
    if numerator == I::WideUInt::ZERO || denominator == I::Wide::ZERO {
        return None;
    }

    let denominator_negative = denominator < I::Wide::ZERO;
    let denominator = denominator.unsigned_abs();
    if numerator_negative != denominator_negative
        || numerator >= denominator
        || denominator >= I::WideUInt::LAST_BIT
    {
        return None;
    }

    let product =
        <I::WideUInt as UIntNumber>::Product::multiply(numerator, SegmentParam::<I>::DENOMINATOR.to_uint());
    let value = I::Wide::from_uint(product.divide_with_rounding(denominator));

    if value <= I::Wide::ZERO || value >= SegmentParam::<I>::DENOMINATOR {
        None
    } else {
        Some(SegmentParam::new(I::from_wide(value)))
    }
}

/// Returns `round(dot(lhs, rhs) / denominator)` when the result is inside
/// the fixed-point unit interval. Products and their signed sum stay in the
/// double-width unsigned representation until after division.
fn unit_dot_ratio_scaled<I: IntNumber>(
    lhs: IntVector<I>,
    rhs: IntVector<I>,
    denominator: I::Wide,
) -> Option<I::Wide> {
    debug_assert!(denominator > I::Wide::ZERO);

    let dot = SignedProduct::multiply(lhs.x, rhs.x).checked_add(SignedProduct::multiply(lhs.y, rhs.y))?;
    if dot.is_negative() {
        return None;
    }
    let magnitude = dot.magnitude();

    let unit_limit = <I::WideUInt as UIntNumber>::Product::multiply(
        denominator.unsigned_abs(),
        FixedScale::<I>::DENOMINATOR.to_uint(),
    );
    if magnitude >= unit_limit {
        return None;
    }

    let quotient = magnitude.divide_with_rounding(denominator.unsigned_abs());
    let value = I::Wide::from_uint(quotient);

    (value > I::Wide::ZERO && value < FixedScale::<I>::DENOMINATOR).then_some(value)
}

fn local_segment_param<I: IntNumber>(t0: SegmentParam<I>, t1: SegmentParam<I>) -> SegmentParam<I> {
    let numerator = t1.value() - t0.value();
    let denominator = SegmentParam::<I>::DENOMINATOR - t0.value();

    SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator))
}

fn solve_self_intersection_params<I: IntNumber>(
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
    fn keeps_monotone_cubic_without_s_shape() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(8, 0),
                IntPoint::new(16, 12),
                IntPoint::new(24, 24),
            ],
        };

        match cubic.normalize_monotone_without_s_shape() {
            CubicSShapeNormalization::NoS(segment) => {
                assert_eq!(segment.control_points, cubic.control_points)
            }
            CubicSShapeNormalization::Pieces(_) => panic!("expected no S split"),
        }
    }

    #[test]
    fn splits_monotone_cubic_at_single_s_shape_inflection() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(8, 0),
                IntPoint::new(16, 24),
                IntPoint::new(24, 24),
            ],
        };

        match cubic.normalize_monotone_without_s_shape() {
            CubicSShapeNormalization::Pieces([first, last]) => {
                assert_eq!(
                    first.control_points,
                    [
                        IntPoint::new(0, 0),
                        IntPoint::new(4, 0),
                        IntPoint::new(8, 6),
                        IntPoint::new(12, 12),
                    ]
                );
                assert_eq!(
                    last.control_points,
                    [
                        IntPoint::new(12, 12),
                        IntPoint::new(16, 18),
                        IntPoint::new(20, 24),
                        IntPoint::new(24, 24),
                    ]
                );
            }
            CubicSShapeNormalization::NoS(_) => panic!("expected S split"),
        }
    }

    #[test]
    fn finds_large_s_shape_root_without_discriminant_overflow() {
        let scale = 2_000_000;
        let cubic = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(5 * scale, 3 * scale),
                IntPoint::new(20 * scale, 22 * scale),
                IntPoint::new(24 * scale, 24 * scale),
            ],
        };

        let root = cubic.s_shape_split_param().unwrap();

        assert!((root.value() - 542_465_139).abs() <= 1);
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

    #[test]
    fn unit_dot_ratio_uses_double_width_products() {
        let lhs = IntVector::<i32>::new(1_000_000, -1_000_000);
        let rhs = IntVector::<i32>::new(1_000_000_000_000_000, -1_000_000_000_000_000);

        let result = unit_dot_ratio_scaled(lhs, rhs, 2_000_000_000_000);

        assert_eq!(result, Some(1_000_000_000));
    }

    #[test]
    fn unit_dot_ratio_handles_opposite_sign_terms() {
        let lhs = IntVector::<i32>::new(1_000_000, 1_000_000);
        let rhs = IntVector::<i32>::new(1_000_000_000_000_000, -999_000_000_000_000);

        let result = unit_dot_ratio_scaled(lhs, rhs, 2_000_000_000_000);

        assert_eq!(result, Some(500_000));
    }

    #[test]
    fn unit_dot_ratio_rejects_values_outside_unit_interval() {
        let lhs = IntVector::<i32>::new(1_000_000, -1_000_000);
        let rhs = IntVector::<i32>::new(2_000_000_000_000_000, -2_000_000_000_000_000);
        let negative_rhs = IntVector::<i32>::new(-2_000_000_000_000_000, 2_000_000_000_000_000);

        assert_eq!(unit_dot_ratio_scaled(lhs, rhs, 2_000_000_000_000), None);
        assert_eq!(unit_dot_ratio_scaled(lhs, negative_rhs, 2_000_000_000_000), None);
    }

    #[test]
    fn normalizes_large_cubic_without_dot_product_overflow() {
        let cubic = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(54_819, 167_472),
                IntPoint::new(6, -7),
                IntPoint::new(-446_637, -116_744),
                IntPoint::new(-8, -2),
            ],
        };

        let _ = cubic.try_segment();
    }

    #[test]
    fn splits_cubic_at_cusp() {
        let cubic = CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
                IntPoint::new(100, 0),
            ],
        };

        let segments = cubic.split_at_cusps();
        let segments = segments.as_slice();

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].control_points[0], cubic.control_points[0]);
        assert_eq!(segments[1].control_points[3], cubic.control_points[3]);
        assert_eq!(segments[0].control_points[3], segments[1].control_points[0]);
    }
}
