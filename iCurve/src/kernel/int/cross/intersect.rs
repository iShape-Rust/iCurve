use crate::int::CurveInt;
use crate::kernel::int::cross::intersector::{
    ContactPoint, SegmentIntersectionBuffer, SegmentIntersector, SplitOptions,
};
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::canonical::PushCanonicalSegment;
use alloc::vec::Vec;

impl<I: CurveInt> Segment<I> {
    pub(crate) fn intersect(self, other: Self) -> Vec<ContactPoint<I>> {
        let mut a_segments = Vec::new();
        let mut b_segments = Vec::new();

        a_segments.push_canonical(self);
        b_segments.push_canonical(other);

        let mut buffer = SegmentIntersectionBuffer::default();
        let mut output = Vec::new();

        for a in a_segments.iter() {
            let a_rect = a.chord().to_rect();
            for b in b_segments.iter() {
                let b_rect = b.chord().to_rect();
                if a_rect.is_intersect_border_exclude(&b_rect) {
                    let intersector = SegmentIntersector::new(*a, *b, SplitOptions::default());
                    for &contact in intersector.intersect_with_buffer(&mut buffer) {
                        if !output.contains(&contact) {
                            output.push(contact);
                        }
                    }
                }
            }
        }

        output
    }
}
#[cfg(test)]
mod tests {
    use crate::kernel::int::cross::intersector::ContactType;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::quad::QuadSegment;
    use crate::kernel::int::curve::segment::Segment;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    fn quarter_circle(center: IntPoint<i32>, left_half: bool) -> Segment<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let (control_points, start_phase, end_phase, direction) = if left_half {
            (
                [
                    IntPoint::new(center.x - 100, center.y),
                    IntPoint::new(center.x - 100, center.y + 100),
                    IntPoint::new(center.x, center.y + 100),
                ],
                ArcPhase { cos: -one, sin: 0 },
                ArcPhase { cos: 0, sin: one },
                ArcDirection::Clockwise,
            )
        } else {
            (
                [
                    IntPoint::new(center.x + 100, center.y),
                    IntPoint::new(center.x + 100, center.y + 100),
                    IntPoint::new(center.x, center.y + 100),
                ],
                ArcPhase { cos: one, sin: 0 },
                ArcPhase { cos: 0, sin: one },
                ArcDirection::CounterClockwise,
            )
        };

        Segment::Arc(ArcSegment {
            ellipse: EllipseFrame {
                center,
                axis_x: ArcVector { x: 100, y: 0 },
                axis_y: ArcVector { x: 0, y: 100 },
            },
            control_points,
            weights: [one, 759_250_125, one],
            start_phase,
            end_phase,
            direction,
        })
    }

    #[test]
    fn test_0() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [940, 899].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_1() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [900, 700].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn intersects_arc_with_line() {
        let arc = quarter_circle(IntPoint::new(0, 0), false);
        let line = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(100, 100)],
        });

        let contacts = arc.intersect(line);

        assert!(contacts.iter().any(|contact| {
            contact.contact_type == ContactType::Cross
                && (contact.point.x - 71).abs() <= 2
                && (contact.point.y - 71).abs() <= 2
        }));
    }

    #[test]
    fn intersects_arc_with_quad_and_cubic() {
        let arc = quarter_circle(IntPoint::new(0, 0), false);
        let quad = Segment::Quad(QuadSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(30, 60),
                IntPoint::new(100, 100),
            ],
        });
        let cubic = Segment::Cubic(CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(20, 60),
                IntPoint::new(80, 40),
                IntPoint::new(100, 100),
            ],
        });

        assert!(
            arc.intersect(quad)
                .iter()
                .any(|contact| contact.contact_type == ContactType::Cross)
        );
        assert!(
            arc.intersect(cubic)
                .iter()
                .any(|contact| contact.contact_type == ContactType::Cross)
        );
    }

    #[test]
    fn intersects_two_arcs() {
        let first = quarter_circle(IntPoint::new(0, 0), false);
        let second = quarter_circle(IntPoint::new(100, 0), true);

        let contacts = first.intersect(second);

        assert!(contacts.iter().any(|contact| {
            contact.contact_type == ContactType::Cross
                && (contact.point.x - 50).abs() <= 2
                && (contact.point.y - 87).abs() <= 2
        }));
    }

    #[test]
    fn test_2() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [596, 795].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_3() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [597, 795].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_4() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [1263, 176].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_5() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [464, 732].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert_eq!(result.len(), 1);
    }
}
