use crate::curve::shape::CurveShape;
use crate::flatten::approx::{LineApproximation, LineApproximationSplit};
use crate::flatten::convert::ShapeToSegments;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use crate::curve::path::CurvePath;

impl<P: FloatPointCompatible> CurvePath<P> {
    pub fn approximate_to_contour(&self, approximation: LineApproximation<P::Scalar>) -> Vec<P> {
        let segments = self.to_normalize_segments(ShapeType::Subject);
        let mut output = Vec::with_capacity(segments.len() + 1);
        output.push(self.start);

        for segment in segments {
            segment
                .segment
                .append_approximated_points(approximation, &mut output);
        }

        output
    }
}

impl<P: FloatPointCompatible> CurveShape<P> {
    pub fn approximate_to_shape(&self, approximation: LineApproximation<P::Scalar>) -> Vec<Vec<P>> {
        self.contours
            .iter()
            .map(|contour| contour.approximate_to_contour(approximation))
            .collect()
    }
}

trait AppendApproximatedPoints<P: FloatPointCompatible> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>);
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for Segment<P::Scalar> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        match self {
            Self::Line(segment) => segment.append_approximated_points(approximation, output),
            Self::Quad(segment) => segment.append_approximated_points(approximation, output),
            Self::Cubic(segment) => segment.append_approximated_points(approximation, output),
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for LineSegment<P::Scalar> {
    fn append_approximated_points(&self, _approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        let point = self.control_points[1];
        output.push(P::from_xy(point.x, point.y));
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for QuadSegment<P::Scalar> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        if self.is_split_required(approximation) {
            let [left, right] = self.split_at(P::Scalar::HALF);
            left.append_approximated_points(approximation, output);
            right.append_approximated_points(approximation, output);
        } else {
            let point = self.control_points[2];
            output.push(P::from_xy(point.x, point.y));
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for CubicSegment<P::Scalar> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        if self.is_split_required(approximation) {
            let [left, right] = self.split_at(P::Scalar::HALF);
            left.append_approximated_points(approximation, output);
            right.append_approximated_points(approximation, output);
        } else {
            let point = self.control_points[3];
            output.push(P::from_xy(point.x, point.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::builder::{CurveError, CurveBuilder};

    fn approximation() -> LineApproximation<f64> {
        LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0001,
        }
    }

    #[test]
    fn approximate_contour_to_points() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .quad_to([0.0, 2.0], [4.0, 0.0])?
            .close_contour()?
            .build_shape()?;

        let contour = shape.contours[0].approximate_to_contour(approximation());

        assert_eq!(contour[0], [0.0, 0.0]);
        assert_eq!(contour[contour.len() - 2], [4.0, 0.0]);
        assert_eq!(*contour.last().unwrap(), [0.0, 0.0]);
        assert!(contour.len() > 3);
        Ok(())
    }

    #[test]
    fn approximate_shape_to_contours() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_contour()?
            .move_to([2.0, 0.0])?
            .line_to([3.0, 0.0])?
            .close_contour()?
            .build_shape()?;

        let contours = shape.approximate_to_shape(approximation());

        assert_eq!(
            contours,
            Vec::from([
                Vec::from([[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]]),
                Vec::from([[2.0, 0.0], [3.0, 0.0], [2.0, 0.0]])
            ])
        );
        Ok(())
    }
}
