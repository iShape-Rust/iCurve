use crate::kernel::int::curve::arc::ArcSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

impl<I: IntNumber> ArcSegment<I> {
    #[inline]
    pub(crate) fn try_segment(self) -> Option<Segment<I>> {
        let [p0, p1, p2] = self.control_points;

        if p0 == p2 {
            None
        } else if Triangle::is_line(p0, p1, p2) {
            LineSegment {
                control_points: [p0, p2],
            }
            .try_segment()
        } else {
            debug_assert!(
                self.is_xy_monotone(),
                "kernel arcs must be split at every world-space extremum"
            );
            Some(Segment::Arc(self))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcVector, EllipseFrame};
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    fn arc(control_points: [IntPoint<i32>; 3]) -> ArcSegment<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;

        ArcSegment {
            ellipse: EllipseFrame {
                center: IntPoint::new(0, 0),
                axis_x: ArcVector { x: 100, y: 0 },
                axis_y: ArcVector { x: 0, y: 100 },
            },
            control_points,
            weights: [one, 759_250_125, one],
            start_phase: ArcPhase { cos: one, sin: 0 },
            end_phase: ArcPhase { cos: 0, sin: one },
            direction: ArcDirection::CounterClockwise,
        }
    }

    #[test]
    fn drops_arc_with_collapsed_chord() {
        let point = IntPoint::new(10, 20);
        let segment = arc([point, IntPoint::new(12, 22), point]).try_segment();

        assert!(segment.is_none());
    }

    #[test]
    fn reduces_collinear_arc_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(5, 5);
        let p2 = IntPoint::new(10, 10);
        let segment = arc([p0, p1, p2]).try_segment();

        match segment {
            Some(Segment::Line(line)) => assert_eq!(line.control_points, [p0, p2]),
            _ => panic!("expected line segment"),
        }
    }

    #[test]
    fn keeps_monotone_non_degenerate_arc() {
        let source = arc([
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
        ]);
        let segment = source.try_segment();

        match segment {
            Some(Segment::Arc(result)) => assert_eq!(result, source),
            _ => panic!("expected arc segment"),
        }
    }
}
