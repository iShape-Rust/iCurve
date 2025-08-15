use crate::int::base::spline::IntSpline;
use crate::int::collision::solver::{Mark, Solver};
use crate::int::collision::x_segment::XOverlap;
use crate::int::math::point::IntPoint;

impl Solver {

    // fn split(&mut self, primary: IntSpline, secondary: IntSpline) {
    //     match self.marks.len() {
    //         0 => return,
    //         1 => {
    //             let m = self.marks.first().unwrap();
    //             match m.overlap {
    //                 XOverlap::Segment(seg) => {}
    //                 XOverlap::Point(pnt) => {}
    //             }
    //         }
    //             // self.single_split(primary, secondary)
    //     }
    //     debug_assert!(!self.marks.is_empty())
    // 
    // 
    // 
    // }

    fn single_split(&mut self, point: IntPoint, primary: IntSpline, secondary: IntSpline) {
        // the most common case





    }

}