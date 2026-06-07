use crate::curve::contour::CurveContour;
use crate::curve::shape::CurveShape;
use crate::flatten::approx::{LineApproximation, LineApproximationSplit};
use crate::flatten::convert::ShapeToSegments;
use crate::flatten::segment::{ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

impl<P: FloatPointCompatible> CurveContour<P> {
    pub fn approximate_to_contour(&self, approximation: LineApproximation<P::Scalar>) -> Vec<P> {
        let segments = self.to_normalize_segments(ShapeType::Subject);
        let mut output = Vec::with_capacity(segments.len() + 1);
        output.push(self.start);

        for segment in segments {
            segment
                .normalized_segment
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

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for NormalizedSegment<P> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        match self {
            Self::Line(segment) => segment.append_approximated_points(approximation, output),
            Self::Quad(segment) => segment.append_approximated_points(approximation, output),
            Self::Cubic(segment) => segment.append_approximated_points(approximation, output),
            Self::Arc(segment) => segment.append_approximated_points(approximation, output),
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for LineSegment<P> {
    fn append_approximated_points(&self, _approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        output.push(self.control_points[1]);
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for QuadSegment<P> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        if self.is_split_required(approximation) {
            let [left, right] = self.split_at_half();
            left.append_approximated_points(approximation, output);
            right.append_approximated_points(approximation, output);
        } else {
            output.push(self.control_points[2]);
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for CubicSegment<P> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        if self.is_split_required(approximation) {
            let [left, right] = self.split_at_half();
            left.append_approximated_points(approximation, output);
            right.append_approximated_points(approximation, output);
        } else {
            output.push(self.control_points[3]);
        }
    }
}

impl<P: FloatPointCompatible> AppendApproximatedPoints<P> for ArcSegment<P> {
    fn append_approximated_points(&self, approximation: LineApproximation<P::Scalar>, output: &mut Vec<P>) {
        let zero = P::Scalar::from_float(0.0);
        let one = P::Scalar::from_float(1.0);
        let min_cos = approximation.min_cos.max(-one).min(one);
        let max_angle = min_cos.acos();
        let sweep_angle = self.sweep_angle.abs();
        let radius = self.radii.x().abs().max(self.radii.y().abs());
        let max_arc_length = radius * sweep_angle;

        if max_angle <= zero
            || sweep_angle <= max_angle
            || max_arc_length * max_arc_length <= approximation.min_segment_sqr_length
        {
            output.push(self.p1);
            return;
        }

        let ratio = sweep_angle / max_angle;
        let mut count = ratio.to_usize();
        if P::Scalar::from_usize(count) < ratio {
            count += 1;
        }
        let count = count.max(1);
        let step = one / P::Scalar::from_usize(count);

        for index in 1..=count {
            let t = if index == count {
                one
            } else {
                P::Scalar::from_usize(index) * step
            };
            output.push(self.point_at(t));
        }
    }
}

trait ArcPoint<P: FloatPointCompatible> {
    fn point_at(&self, t: P::Scalar) -> P;
}

impl<P: FloatPointCompatible> ArcPoint<P> for ArcSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        let angle = self.start_angle + self.sweep_angle * t;
        let x = self.radii.x() * angle.cos();
        let y = self.radii.y() * angle.sin();
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        P::from_xy(
            self.center.x() + x * cos - y * sin,
            self.center.y() + x * sin + y * cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::arc::EllipticArc;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};

    fn approximation() -> LineApproximation<f64> {
        LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0001,
        }
    }

    #[test]
    fn approximate_contour_to_points() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .quad_to([0.0, 2.0], [4.0, 0.0])?
            .close_with_line()?
            .build()?;

        let contour = shape.contours[0].approximate_to_contour(approximation());

        assert_eq!(contour[0], [0.0, 0.0]);
        assert_eq!(contour[contour.len() - 2], [4.0, 0.0]);
        assert_eq!(*contour.last().unwrap(), [0.0, 0.0]);
        assert!(contour.len() > 3);
        Ok(())
    }

    #[test]
    fn approximate_shape_to_contours() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_with_line()?
            .move_to([2.0, 0.0])?
            .line_to([3.0, 0.0])?
            .close_with_line()?
            .build()?;

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

    #[test]
    fn approximate_arc_to_points() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([1.0, 0.0])?
            .arc_to(EllipticArc {
                center: [0.0, 0.0],
                radii: [1.0, 1.0],
                rotation: 0.0,
                start_angle: 0.0,
                sweep_angle: core::f64::consts::FRAC_PI_2,
            })?
            .close_with_line()?
            .build()?;

        let contour = shape.contours[0].approximate_to_contour(LineApproximation {
            min_cos: core::f64::consts::FRAC_PI_8.cos(),
            min_segment_sqr_length: 0.0,
        });

        assert_eq!(contour.len(), 6);
        assert_eq!(contour[0], [1.0, 0.0]);
        assert!(contour[4][0].abs() < 0.000001);
        assert!((contour[4][1] - 1.0).abs() < 0.000001);
        assert_eq!(contour[5], [1.0, 0.0]);
        Ok(())
    }
}
