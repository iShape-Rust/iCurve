use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

impl<I: IntNumber> QuadSegment<I> {
    #[inline]
    pub(crate) fn try_segment(self) -> Option<Segment<I>> {
        let [p0, p1, p2] = self.control_points;

        // Closed quadratic normalizes to an out-and-back spike.
        if p0 == p2 {
            None
        } else if Triangle::is_line(p0, p1, p2) {
            // Collinear quadratic contributes the same edge as its chord.
            LineSegment {
                control_points: [p0, p2],
            }
            .try_segment()
        } else {
            Some(Segment::Quad(self))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn drops_closed_quad_spike() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p0],
        };

        let segment = quad.try_segment();

        assert!(segment.is_none());
    }

    #[test]
    fn reduces_collinear_quad_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 0);
        let p2 = IntPoint::new(2, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p2],
        };

        let segment = quad.try_segment();

        match segment {
            Some(Segment::Line(segment)) => assert_eq!(segment.control_points, [p0, p2]),
            _ => panic!("expected line segment"),
        }
    }

    #[test]
    fn keeps_non_degenerate_quad() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 1);
        let p2 = IntPoint::new(2, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p2],
        };

        let segment = quad.try_segment();

        match segment {
            Some(Segment::Quad(segment)) => assert_eq!(segment.control_points, [p0, p1, p2]),
            _ => panic!("expected quad segment"),
        }
    }
}
