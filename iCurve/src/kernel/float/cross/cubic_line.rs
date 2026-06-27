use crate::kernel::float::cross::contact::ContactPoint;
use crate::kernel::float::cross::solver::Solver;
use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_cubic_and_line(
        &mut self,
        cubic: FloatCubicSegment<T>,
        line: FloatLineSegment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) {
        let cubic_rect = FloatRect::with_points(&cubic.control_points);
        let line_rect = FloatRect::with_points(&line.control_points);
    }
}
