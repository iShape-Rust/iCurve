use crate::int::curve::path::CurvePath;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurveShape<I: IntNumber> {
    pub contours: Vec<CurvePath<I>>,
}

impl<I: IntNumber> CurveShape<I> {
    /// Creates a shape from closed contours.
    pub fn new(contours: Vec<CurvePath<I>>) -> Self {
        Self { contours }
    }

    /// Creates a shape containing one closed contour.
    pub fn from_path(path: CurvePath<I>) -> Self {
        Self {
            contours: alloc::vec![path],
        }
    }

    /// Returns the total number of curve segments in the shape.
    pub fn segment_count(&self) -> usize {
        self.contours.iter().map(|path| path.segments.len()).sum()
    }
}
