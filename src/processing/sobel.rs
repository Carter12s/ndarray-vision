use crate::core::*;
use crate::processing::*;
use core::ops::Neg;
use ndarray::{prelude::*, DataMut, OwnedRepr};
use num_traits::{cast::FromPrimitive, real::Real, Num, NumAssignOps};
use std::marker::Sized;

/// Runs the sobel operator on an image
pub trait SobelExt
where
    Self: Sized,
{
    /// Type to output
    type Output;
    /// Returns the magnitude output of the sobel - an image of only lines
    fn apply_sobel(&self) -> Result<Self::Output, Error>;

    /// Returns the magntitude and rotation outputs for use in other algorithms
    /// like the Canny edge detector. Rotation is in radians
    fn full_sobel(&self) -> Result<(Self::Output, Self::Output), Error>;
}

fn get_edge_images<T, U>(mat: &ArrayBase<U, Ix3>) -> Result<(Array3<T>, Array3<T>), Error>
where
    U: DataMut<Elem = T>,
    T: Copy + Clone + Num + NumAssignOps + Neg<Output = T> + FromPrimitive + Real,
{
    let v_temp: Array3<T> = SobelFilter::build_with_params(Orientation::Vertical).unwrap();
    let h_temp: Array3<T> = SobelFilter::build_with_params(Orientation::Horizontal).unwrap();
    let shape = (v_temp.shape()[0], v_temp.shape()[1], mat.shape()[2]);
    let h_kernel = Array3::<T>::from_shape_fn(shape, |(i, j, _)| h_temp[[i, j, 0]]);
    let v_kernel = Array3::<T>::from_shape_fn(shape, |(i, j, _)| v_temp[[i, j, 0]]);

    let h_deriv = mat.conv2d(h_kernel.view())?;
    let v_deriv = mat.conv2d(v_kernel.view())?;

    Ok((h_deriv, v_deriv))
}

impl<T, U> SobelExt for ArrayBase<U, Ix3>
where
    U: DataMut<Elem = T>,
    T: Copy + Clone + Num + NumAssignOps + Neg<Output = T> + FromPrimitive + Real,
{
    type Output = ArrayBase<OwnedRepr<T>, Ix3>;

    fn apply_sobel(&self) -> Result<Self::Output, Error> {
        let (h_deriv, v_deriv) = get_edge_images(self)?;

        let h_deriv = h_deriv.mapv(|x| x.powi(2));
        let v_deriv = v_deriv.mapv(|x| x.powi(2));

        let mut result = h_deriv + v_deriv;
        result.mapv_inplace(|x| x.sqrt());

        // squash values above 1.0
        result.mapv_inplace(|x| if x > T::one() { T::one() } else { x });

        Ok(result)
    }

    fn full_sobel(&self) -> Result<(Self::Output, Self::Output), Error> {
        let (h_deriv, v_deriv) = get_edge_images(self)?;

        let mut magnitude = h_deriv.mapv(|x| x.powi(2)) + v_deriv.mapv(|x| x.powi(2));
        magnitude.mapv_inplace(|x| x.sqrt());
        magnitude.mapv_inplace(|x| if x > T::one() { T::one() } else { x });

        let mut rotation = v_deriv / h_deriv;
        rotation.mapv_inplace(|x| x.atan());

        Ok((magnitude, rotation))
    }
}

impl<T, U, C> SobelExt for ImageBase<U, C>
where
    U: DataMut<Elem = T>,
    T: Copy + Clone + Num + NumAssignOps + Neg<Output = T> + FromPrimitive + Real,
    C: ColourModel,
{
    type Output = Image<T, C>;

    fn apply_sobel(&self) -> Result<Self::Output, Error> {
        let data = self.data.apply_sobel()?;
        Ok(Image::from_data(data))
    }

    fn full_sobel(&self) -> Result<(Self::Output, Self::Output), Error> {
        self.data
            .full_sobel()
            .map(|(m, r)| (Image::from_data(m), Image::from_data(r)))
    }
}
