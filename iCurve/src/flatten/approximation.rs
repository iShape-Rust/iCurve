use crate::curve::path::CurvePath;
use crate::curve::shape::CurveShape;
use crate::flatten::condition::{FlatCondition, FlatParams};
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use crate::normalization::curve::CurveToSegments;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;

pub trait LineApproximation<T: FloatNumber> {
    type Output;
    fn approximate_with_adapter<I: IntNumber>(
        &self,
        approximation: FlatParams<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Self::Output;
}

impl<P: FloatPointCompatible> LineApproximation<P::Scalar> for CurveShape<P> {
    type Output = Vec<Vec<P>>;

    fn approximate_with_adapter<I: IntNumber>(
        &self,
        params: FlatParams<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Self::Output {
        self.contours
            .iter()
            .map(|contour| contour.approximate_with_adapter(params, adapter))
            .collect()
    }
}

impl<P: FloatPointCompatible> LineApproximation<P::Scalar> for CurvePath<P> {
    type Output = Vec<P>;
    fn approximate_with_adapter<I: IntNumber>(
        &self,
        params: FlatParams<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Self::Output {
        let Ok(segments) = self.try_to_normalize_segments_with_adapter(adapter) else {
            return Vec::new();
        };
        let mut output = Vec::with_capacity(segments.len() + 1);
        output.push(self.start);

        for segment in segments {
            segment.append_approximated_points(params, &mut output);
        }

        output
    }
}

trait AppendApproximatedPoints<P: FloatPointCompatible> {
    fn append_approximated_points(&self, approximation: FlatParams<P::Scalar>, output: &mut Vec<P>);
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for Segment<P::Scalar> {
    fn append_approximated_points(&self, approximation: FlatParams<P::Scalar>, output: &mut Vec<P>) {
        match self {
            Self::Line(segment) => segment.append_approximated_points(approximation, output),
            Self::Quad(segment) => segment.append_approximated_points(approximation, output),
            Self::Cubic(segment) => segment.append_approximated_points(approximation, output),
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for LineSegment<P::Scalar> {
    fn append_approximated_points(&self, _approximation: FlatParams<P::Scalar>, output: &mut Vec<P>) {
        let point = self.control_points[1];
        output.push(P::from_xy(point.x, point.y));
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for QuadSegment<P::Scalar> {
    fn append_approximated_points(&self, approximation: FlatParams<P::Scalar>, output: &mut Vec<P>) {
        if self.is_not_flat(approximation) {
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
    fn append_approximated_points(&self, approximation: FlatParams<P::Scalar>, output: &mut Vec<P>) {
        if self.is_not_flat(approximation) {
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
    use crate::curve::builder::{CurveBuilder, CurveError};

    fn approximation() -> FlatParams<f64> {
        FlatParams {
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

        let adapter: FloatPointAdapter<_, i32> = FloatPointAdapter::with_radius_and_scale(100.0, 1000.0);
        let contour = shape.contours[0].approximate_with_adapter(approximation(), &adapter);

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

        let adapter: FloatPointAdapter<_, i32> = FloatPointAdapter::with_radius_and_scale(100.0, 1000.0);
        let contours = shape.approximate_with_adapter(approximation(), &adapter);

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
