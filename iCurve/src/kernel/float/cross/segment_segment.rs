use crate::kernel::float::cross::contact::ContactPoint;
use crate::kernel::float::cross::solver::Solver;
use crate::kernel::float::curve::segment::FloatSegment;
use alloc::vec::Vec;
use core::mem;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> Solver<T> {
    pub fn intersect_segment_and_segment(
        &mut self,
        segment0: FloatSegment<T>,
        segment1: FloatSegment<T>,
        output: &mut Vec<ContactPoint<T>>,
    ) {
        match (segment0, segment1) {
            (FloatSegment::Line(line0), FloatSegment::Line(line1)) => {
                self.intersect_line_and_line(line0, line1, output);
            }
            (FloatSegment::Line(line), FloatSegment::Quad(quad)) => {
                let count = output.len();
                self.intersect_quad_and_line(quad, line, output);
                Self::swap_contact_params(&mut output[count..]);
            }
            (FloatSegment::Line(_), FloatSegment::Cubic(_)) => {
                panic!("line-cubic intersection is not implemented");
            }
            (FloatSegment::Quad(quad), FloatSegment::Line(line)) => {
                self.intersect_quad_and_line(quad, line, output);
            }
            (FloatSegment::Quad(quad0), FloatSegment::Quad(quad1)) => {
                self.intersect_quad_and_quad(quad0, quad1, output);
            }
            (FloatSegment::Quad(_), FloatSegment::Cubic(_)) => {
                panic!("quad-cubic intersection is not implemented");
            }
            (FloatSegment::Cubic(_), FloatSegment::Line(_)) => {
                panic!("cubic-line intersection is not implemented");
            }
            (FloatSegment::Cubic(_), FloatSegment::Quad(_)) => {
                panic!("cubic-quad intersection is not implemented");
            }
            (FloatSegment::Cubic(_), FloatSegment::Cubic(_)) => {
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
