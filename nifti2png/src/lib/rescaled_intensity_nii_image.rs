use crate::{
    error_ty::ErrorTy, nii_image::NiiImage,
    rescaled_intensity_nii_slice::RescaledIntensityNiiSlice, SECONDARY_DIMS,
};
use pyo3::prelude::*;

pub(crate) struct RescaledIntensityNiiImage<'a>(NiiImage<'a>);

impl<'a> RescaledIntensityNiiImage<'a> {
    pub(crate) fn new(nii_image: NiiImage<'a>) -> Self {
        Self(nii_image)
    }

    pub(crate) fn dim(&self, i: usize) -> isize {
        self.0.dims[i]
    }

    pub(crate) fn get_slice(
        &self,
        py: Python<'a>,
        index: [isize; SECONDARY_DIMS],
    ) -> Result<RescaledIntensityNiiSlice<'a>, ErrorTy> {
        self.0
            .get_slice(py, index)
            .map(RescaledIntensityNiiSlice::new)
    }
}
