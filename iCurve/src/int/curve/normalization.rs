use crate::kernel::int::curve::{
    cubic::CubicSegment as KernelCubicSegment, line::LineSegment as KernelLineSegment,
    quad::QuadSegment as KernelQuadSegment, segment::Segment as KernelSegment,
};
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizedSegment<I: IntNumber> {
    Line([IntPoint<I>; 2]),
    Quad([IntPoint<I>; 3]),
    Cubic([IntPoint<I>; 4]),
}

#[inline]
pub fn cubic_self_intersection<I: IntNumber>(control_points: [IntPoint<I>; 4]) -> Option<IntPoint<I>> {
    let cubic = KernelCubicSegment { control_points };
    cubic.self_intersection_point()
}

pub fn normalize_cubic<I: IntNumber>(control_points: [IntPoint<I>; 4]) -> Vec<NormalizedSegment<I>> {
    let cubic = KernelCubicSegment { control_points };

    cubic
        .try_segment()
        .as_slice()
        .iter()
        .copied()
        .map(normalized_segment)
        .collect()
}

fn normalized_segment<I: IntNumber>(segment: KernelSegment<I>) -> NormalizedSegment<I> {
    match segment {
        KernelSegment::Line(KernelLineSegment { control_points }) => NormalizedSegment::Line(control_points),
        KernelSegment::Quad(KernelQuadSegment { control_points }) => NormalizedSegment::Quad(control_points),
        KernelSegment::Cubic(KernelCubicSegment { control_points }) => {
            NormalizedSegment::Cubic(control_points)
        }
    }
}
