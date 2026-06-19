use crate::kernel::cross::contact::ContactPoint;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_cubic_and_line(
        &mut self,
        cubic: CubicSegment<T>,
        line: LineSegment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) {
        let cubic_rect = FloatRect::with_points(&cubic.control_points);
        let line_rect = FloatRect::with_points(&line.control_points);
    }
}
