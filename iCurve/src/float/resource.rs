//! Curve resources accepted by float overlay operations.

use crate::float::curve::path::CurvePath;
use crate::float::curve::shape::CurveShape;
use alloc::boxed::Box;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

pub(crate) mod private {
    use super::{CurvePath, CurveShape, FloatPointCompatible};

    pub trait SealedCurveResource<P>
    where
        P: FloatPointCompatible,
    {
        type ResourceIter<'a>: Iterator<Item = &'a CurvePath<P>>
        where
            P: 'a,
            Self: 'a;

        fn iter_paths(&self) -> Self::ResourceIter<'_>;
    }

    pub struct CurveShapesResourceIter<'a, P: FloatPointCompatible> {
        shapes: core::slice::Iter<'a, CurveShape<P>>,
        paths: Option<core::slice::Iter<'a, CurvePath<P>>>,
    }

    impl<'a, P: FloatPointCompatible> CurveShapesResourceIter<'a, P> {
        #[inline]
        pub(super) fn new(shapes: &'a [CurveShape<P>]) -> Self {
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

    pub struct CurvePathRefsResourceIter<'a, 'r, P: FloatPointCompatible>
    where
        'r: 'a,
    {
        paths: core::slice::Iter<'a, &'r CurvePath<P>>,
    }

    impl<'a, 'r, P> CurvePathRefsResourceIter<'a, 'r, P>
    where
        'r: 'a,
        P: FloatPointCompatible,
    {
        #[inline]
        pub(super) fn new(paths: &'a [&'r CurvePath<P>]) -> Self {
            Self { paths: paths.iter() }
        }
    }

    impl<'a, 'r, P> Iterator for CurvePathRefsResourceIter<'a, 'r, P>
    where
        'r: 'a,
        P: FloatPointCompatible,
    {
        type Item = &'a CurvePath<P>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.paths.next().copied()
        }
    }

    pub struct CurveShapeRefsResourceIter<'a, 'r, P: FloatPointCompatible>
    where
        'r: 'a,
    {
        shapes: core::slice::Iter<'a, &'r CurveShape<P>>,
        paths: Option<core::slice::Iter<'a, CurvePath<P>>>,
    }

    impl<'a, 'r, P> CurveShapeRefsResourceIter<'a, 'r, P>
    where
        'r: 'a,
        P: FloatPointCompatible,
    {
        #[inline]
        pub(super) fn new(shapes: &'a [&'r CurveShape<P>]) -> Self {
            Self {
                shapes: shapes.iter(),
                paths: None,
            }
        }
    }

    impl<'a, 'r, P> Iterator for CurveShapeRefsResourceIter<'a, 'r, P>
    where
        'r: 'a,
        P: FloatPointCompatible,
    {
        type Item = &'a CurvePath<P>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(path) = self.paths.as_mut().and_then(Iterator::next) {
                    return Some(path);
                }

                let shape: &'a CurveShape<P> = *self.shapes.next()?;
                self.paths = Some(shape.contours.iter());
            }
        }
    }
}

/// Borrowed source of curve paths.
///
/// A resource may represent one path, one shape, or a collection of paths or
/// shapes. All paths yielded by a resource belong to the same overlay operand.
///
/// This trait is sealed. Supported resources are [`CurvePath`], [`CurveShape`],
/// their slices, arrays, and [`Vec`] collections, including collections of
/// references. A [`Box`] around a supported resource is also accepted.
pub trait CurveResource<P>: private::SealedCurveResource<P>
where
    P: FloatPointCompatible,
{
}

impl<P, R> CurveResource<P> for R
where
    P: FloatPointCompatible,
    R: private::SealedCurveResource<P> + ?Sized,
{
}

impl<P, R> private::SealedCurveResource<P> for &R
where
    P: FloatPointCompatible,
    R: private::SealedCurveResource<P> + ?Sized,
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

impl<P, R> private::SealedCurveResource<P> for Box<R>
where
    P: FloatPointCompatible,
    R: private::SealedCurveResource<P> + ?Sized,
{
    type ResourceIter<'a>
        = R::ResourceIter<'a>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        (**self).iter_paths()
    }
}

impl<P> private::SealedCurveResource<P> for CurvePath<P>
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

impl<P> private::SealedCurveResource<P> for CurveShape<P>
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

impl<P> private::SealedCurveResource<P> for [CurvePath<P>]
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

impl<P, const N: usize> private::SealedCurveResource<P> for [CurvePath<P>; N]
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

impl<P> private::SealedCurveResource<P> for Vec<CurvePath<P>>
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

impl<'r, P> private::SealedCurveResource<P> for [&'r CurvePath<P>]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurvePathRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurvePathRefsResourceIter::new(self)
    }
}

impl<'r, P, const N: usize> private::SealedCurveResource<P> for [&'r CurvePath<P>; N]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurvePathRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurvePathRefsResourceIter::new(self)
    }
}

impl<'r, P> private::SealedCurveResource<P> for Vec<&'r CurvePath<P>>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurvePathRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurvePathRefsResourceIter::new(self)
    }
}

impl<P> private::SealedCurveResource<P> for [CurveShape<P>]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapesResourceIter::new(self)
    }
}

impl<P, const N: usize> private::SealedCurveResource<P> for [CurveShape<P>; N]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapesResourceIter::new(self)
    }
}

impl<P> private::SealedCurveResource<P> for Vec<CurveShape<P>>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapesResourceIter<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapesResourceIter::new(self)
    }
}

impl<'r, P> private::SealedCurveResource<P> for [&'r CurveShape<P>]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapeRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapeRefsResourceIter::new(self)
    }
}

impl<'r, P, const N: usize> private::SealedCurveResource<P> for [&'r CurveShape<P>; N]
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapeRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapeRefsResourceIter::new(self)
    }
}

impl<'r, P> private::SealedCurveResource<P> for Vec<&'r CurveShape<P>>
where
    P: FloatPointCompatible,
{
    type ResourceIter<'a>
        = private::CurveShapeRefsResourceIter<'a, 'r, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_paths(&self) -> Self::ResourceIter<'_> {
        private::CurveShapeRefsResourceIter::new(self)
    }
}

pub(crate) fn resource_bounds<P, R>(resource: &R) -> Option<FloatRect<P::Scalar>>
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    resource
        .iter_paths()
        .map(CurvePath::bounds)
        .reduce(FloatRect::with_rects)
}

#[cfg(test)]
mod tests {
    use super::private::SealedCurveResource as _;
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

    #[test]
    fn accepts_owner_wrappers_and_reference_collections() {
        let first = rectangle(0.0);
        let second = rectangle(2.0);
        let shape_refs = [&first, &second];
        let path_refs = [&first.contours[0], &second.contours[0]];

        assert_eq!(shape_refs.iter_paths().count(), 2);
        assert_eq!(shape_refs.as_slice().iter_paths().count(), 2);
        assert_eq!(alloc::vec![&first, &second].iter_paths().count(), 2);
        assert_eq!(path_refs.iter_paths().count(), 2);
        assert_eq!(path_refs.as_slice().iter_paths().count(), 2);
        assert_eq!(alloc::vec![path_refs[0], path_refs[1]].iter_paths().count(), 2);

        assert_eq!(Box::new(first).iter_paths().count(), 1);
    }
}
