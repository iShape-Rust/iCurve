use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use crate::kernel::cross::point::CrossPoint;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_quad_and_line<>(&mut self, quad: QuadSegment<T>, line: LineSegment<T>, output: &mut Vec<CrossPoint<T>>) {
        let quad_rect = FloatRect::with_points(&quad.control_points);
        let line_rect = FloatRect::with_points(&line.control_points);

    }
}