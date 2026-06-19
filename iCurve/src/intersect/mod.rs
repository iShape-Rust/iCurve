use crate::curve::path::CurvePath;
use crate::curve::rect::CurveToFloatRect;
use crate::kernel::cross::solver::Solver;
use crate::kernel::curve::segment::Segment;
use crate::normalization::curve::CurveToSegments;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;

impl<P: FloatPointCompatible> CurvePath<P> {
    pub fn try_intersection_points(
        &self,
        other: &Self,
    ) -> Result<Vec<FloatPoint<P::Scalar>>, FloatPointAdapterRangeError> {
        self.try_intersection_points_as::<i32>(other)
    }

    pub fn try_intersection_points_as<I: IntNumber>(
        &self,
        other: &Self,
    ) -> Result<Vec<FloatPoint<P::Scalar>>, FloatPointAdapterRangeError> {
        let rect = FloatRect::with_optional_rects(self.float_rect(), other.float_rect())
            .unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<FloatPoint<P::Scalar>, I>::new(rect);
        self.try_intersection_points_with_adapter(other, &adapter)
    }

    pub fn try_intersection_points_with_adapter<I: IntNumber>(
        &self,
        other: &Self,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Result<Vec<FloatPoint<P::Scalar>>, FloatPointAdapterRangeError> {
        let segments0 = self.try_to_normalize_segments_with_adapter(adapter)?;
        let segments1 = other.try_to_normalize_segments_with_adapter(adapter)?;

        let mut result = Vec::new();
        let grid_size = P::Scalar::ONE / adapter.dir_scale();
        extend_intersection_points(&segments0, &segments1, grid_size, &mut result);
        Ok(result)
    }
}

fn extend_intersection_points<T: FloatNumber>(
    segments0: &[Segment<T>],
    segments1: &[Segment<T>],
    grid_size: T,
    output: &mut Vec<FloatPoint<T>>,
) {
    let mut solver = Solver::with_grid_size(grid_size);
    let mut points = Vec::new();

    for segment0 in segments0 {
        for segment1 in segments1 {
            points.clear();
            solver.intersect_segment_and_segment(*segment0, *segment1, &mut points);
            output.extend(points.iter().map(|point| point.point));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::curve::builder::{CurveBuilder, CurveError};

    #[test]
    fn line_paths_intersection_points() -> Result<(), CurveError> {
        let path0 = CurveBuilder::new()
            .move_to([0.0_f64, 0.0])?
            .line_to([10.0, 10.0])?
            .build_path()?;
        let path1 = CurveBuilder::new()
            .move_to([0.0_f64, 10.0])?
            .line_to([10.0, 0.0])?
            .build_path()?;

        let points = path0.try_intersection_points(&path1).unwrap();

        assert_eq!(points.len(), 1);
        assert_eq!(points[0].x, 5.0);
        assert_eq!(points[0].y, 5.0);

        Ok(())
    }
}
