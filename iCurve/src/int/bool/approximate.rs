use crate::int::CurveInt;
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::overlay::CurveOverlayOptions;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

#[derive(Clone, Copy)]
struct ApproximationItem<I: CurveInt> {
    edge: CurveEdge<I>,
    depth: u32,
}

pub(crate) struct CurveApproximator<I: CurveInt> {
    output: Vec<CurveEdge<I>>,
    stack: Vec<ApproximationItem<I>>,
}

impl<I: CurveInt> CurveApproximator<I> {
    pub(crate) fn new() -> Self {
        Self {
            output: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub(crate) fn approximate(&mut self, edges: &mut Vec<CurveEdge<I>>, options: CurveOverlayOptions) {
        if edges.is_empty() {
            return;
        }

        self.output.clear();
        self.stack.clear();
        self.output.reserve(edges.len());

        for &edge in edges.iter() {
            self.stack.push(ApproximationItem { edge, depth: 0 });

            while let Some(item) = self.stack.pop() {
                let chord = item.edge.curve.chord();
                if chord.is_zero_length() {
                    continue;
                }

                if item.depth >= options.max_approximation_depth
                    || Self::is_small(chord.sqr_length(), options.min_chord_length_power)
                    || item.edge.curve.is_nearly_linear(options.angle_tolerance_power)
                {
                    self.output.push(item.edge);
                    continue;
                }

                let local_middle = SegmentParam::half();
                let point = item.edge.curve.point_at(local_middle);
                let [left, right] = item.edge.curve.split_at_point(local_middle, point);
                let source_middle = item.edge.start_param.mid(item.edge.end_param);
                let next_depth = item.depth + 1;

                // Stack order is reversed so output keeps the source direction.
                if !right.chord().is_zero_length() {
                    self.stack.push(ApproximationItem {
                        edge: CurveEdge::new(right, item.edge.curve_id, source_middle, item.edge.end_param),
                        depth: next_depth,
                    });
                }
                if !left.chord().is_zero_length() {
                    self.stack.push(ApproximationItem {
                        edge: CurveEdge::new(left, item.edge.curve_id, item.edge.start_param, source_middle),
                        depth: next_depth,
                    });
                }
            }
        }

        core::mem::swap(edges, &mut self.output);
    }

    #[inline]
    fn is_small(sqr_length: I::Wide, min_length_power: u32) -> bool {
        if sqr_length <= I::Wide::ONE {
            return true;
        }
        sqr_length.ilog2() < min_length_power.saturating_mul(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::source::CurveId;
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn keeps_long_lines_as_single_chords() {
        let line = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(10_000, 0)],
        });
        let mut edges = vec![CurveEdge::full(line, CurveId(0))];

        CurveApproximator::new().approximate(&mut edges, CurveOverlayOptions::default());

        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn subdivides_curved_segment_and_preserves_source_range() {
        let cubic = Segment::Cubic(CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 100),
                IntPoint::new(100, 100),
                IntPoint::new(100, 0),
            ],
        });
        let mut edges = vec![CurveEdge::full(cubic, CurveId(0))];

        CurveApproximator::new().approximate(&mut edges, CurveOverlayOptions::default());

        assert!(edges.len() > 1);
        assert_eq!(edges.first().unwrap().start_param, SegmentParam::new(0));
        assert_eq!(
            edges.last().unwrap().end_param.value(),
            SegmentParam::<i32>::DENOMINATOR
        );
        for pair in edges.windows(2) {
            assert_eq!(pair[0].end_param, pair[1].start_param);
        }
    }

    #[test]
    fn length_limit_terminates_non_linear_subdivision() {
        let cubic = Segment::Cubic(CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 8),
                IntPoint::new(8, 8),
                IntPoint::new(8, 0),
            ],
        });
        let mut edges = vec![CurveEdge::full(cubic, CurveId(0))];
        let options = CurveOverlayOptions {
            min_chord_length_power: 4,
            angle_tolerance_power: 30,
            max_approximation_depth: 16,
            ..Default::default()
        };

        CurveApproximator::new().approximate(&mut edges, options);

        assert_eq!(edges.len(), 1);
    }
}
