use crate::kernel::cross::contact::ContactPoint;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::segment::Segment;
use alloc::vec::Vec;
use core::mem;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_segment_and_segment(
        &mut self,
        segment0: Segment<T>,
        segment1: Segment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) {
        match (segment0, segment1) {
            (Segment::Line(line0), Segment::Line(line1)) => {
                self.intersect_line_and_line(line0, line1, output);
            }
            (Segment::Line(line), Segment::Quad(quad)) => {
                let count = output.len();
                self.intersect_quad_and_line(quad, line, output);
                Self::swap_contact_params(&mut output[count..]);
            }
            (Segment::Line(_), Segment::Cubic(_)) => {
                panic!("line-cubic intersection is not implemented");
            }
            (Segment::Quad(quad), Segment::Line(line)) => {
                self.intersect_quad_and_line(quad, line, output);
            }
            (Segment::Quad(quad0), Segment::Quad(quad1)) => {
                self.intersect_quad_and_quad(quad0, quad1, output);
            }
            (Segment::Quad(_), Segment::Cubic(_)) => {
                panic!("quad-cubic intersection is not implemented");
            }
            (Segment::Cubic(_), Segment::Line(_)) => {
                panic!("cubic-line intersection is not implemented");
            }
            (Segment::Cubic(_), Segment::Quad(_)) => {
                panic!("cubic-quad intersection is not implemented");
            }
            (Segment::Cubic(_), Segment::Cubic(_)) => {
                panic!("cubic-cubic intersection is not implemented");
            }
        }
    }

    fn swap_contact_params(points: &mut [ContactPoint<T>]) {
        for point in points {
            mem::swap(&mut point.t0, &mut point.t1);
        }
    }
}
