//! Curve resources accepted by float overlay operations.

use crate::float::curve::path::CurvePath;
use crate::float::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

/// Borrowed source of curve paths.
///
/// A resource may represent one path, one shape, or a collection of paths or
/// shapes. All paths yielded by a resource belong to the same overlay operand.
pub trait CurveResource<P>
where
    P: FloatPointCompatible,
{
    /// Iterator over the paths contained in this resource.
    type ResourceIter<'a>: Iterator<Item = &'a CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    /// Iterates over all curve paths in this resource.
    fn iter_paths(&self) -> Self::ResourceIter<'_>;
}

impl<P, R> CurveResource<P> for &R
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    type ResourceIter<'a>
        = R::ResourceIter<'a>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        (*self).iter_paths()
    }
}

impl<P> CurveResource<P> for CurvePath<P>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = core::iter::Once<&'a CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        core::iter::once(self)
    }
}

impl<P> CurveResource<P> for CurveShape<P>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = core::slice::Iter<'a, CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        self.contours.iter()
    }
}

impl<P> CurveResource<P> for [CurvePath<P>]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = core::slice::Iter<'a, CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        self.iter()
    }
}

impl<P, const N: usize> CurveResource<P> for [CurvePath<P>; N]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = core::slice::Iter<'a, CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        self.iter()
    }
}

impl<P> CurveResource<P> for Vec<CurvePath<P>>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = core::slice::Iter<'a, CurvePath<P>>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        self.iter()
    }
}

/// Iterator that flattens a collection of curve shapes into their paths.
pub struct CurveShapesResourceIter<'a, P: FloatPointCompatible> {
    shapes: core::slice::Iter<'a, CurveShape<P>>,
    paths: Option<core::slice::Iter<'a, CurvePath<P>>>,
}

impl<'a, P: FloatPointCompatible> CurveShapesResourceIter<'a, P> {
    #[inline]
    fn new(shapes: &'a [CurveShape<P>]) -> Self {
        Self {
            shapes: shapes.iter(),
            paths: None,
        }
    }
}

impl<'a, P: FloatPointCompatible> Iterator for CurveShapesResourceIter<'a, P> {
    type Item = &'a CurvePath<P>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(path) = self.paths.as_mut().and_then(Iterator::next) {
                return Some(path);
            }

            let shape = self.shapes.next()?;
            self.paths = Some(shape.contours.iter());
        }
    }
}

impl<P> CurveResource<P> for [CurveShape<P>]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        CurveShapesResourceIter::new(self)
    }
}

impl<P, const N: usize> CurveResource<P> for [CurveShape<P>; N]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        CurveShapesResourceIter::new(self)
    }
}

impl<P> CurveResource<P> for Vec<CurveShape<P>>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        CurveShapesResourceIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CurveBuilder;

    fn rectangle(x: f64) -> CurveShape<[f64; 2]> {
        CurveBuilder::new()
            .move_to([x, 0.0])
            .unwrap()
            .line_to([x + 1.0, 0.0])
            .unwrap()
            .line_to([x + 1.0, 1.0])
            .unwrap()
            .close_contour()
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    fn iterates_one_path_and_one_shape() {
        let shape = rectangle(0.0);
        let path = &shape.contours()[0];

        assert_eq!(path.iter_paths().count(), 1);
        assert_eq!(shape.iter_paths().count(), 1);
    }

    #[test]
    fn flattens_shape_collections() {
        let shapes = alloc::vec![rectangle(0.0), rectangle(2.0)];

        assert_eq!(shapes.iter_paths().count(), 2);
        assert_eq!(shapes.as_slice().iter_paths().count(), 2);
    }
}
