use arrayvec::ArrayVec;
use pyo3::{
    prelude::*,
    types::{PyDict, PySlice},
};

use crate::{error_ty::ErrorTy, rescaled_intensity_nii_image::RescaledIntensityNiiImage, MAX_DIMS, SECONDARY_DIMS};

/// Loaded NIFTI image
pub(crate) struct NiiImage<'a> {
    pub(crate) fdata: &'a PyAny,
    pub(crate) dims: ArrayVec<isize, MAX_DIMS>,
}

impl<'a> NiiImage<'a> {
    pub(crate) fn rescale_intensity_to_unit_interval(
        mut self,
        py: Python<'a>,
        exposure: &'a PyAny,
        minmax: Option<(u64, u64)>,
    ) -> Result<RescaledIntensityNiiImage<'a>, ErrorTy> {
        // Rescale to 0..255 uint8
        self.fdata = exposure.call_method(
            "rescale_intensity",
            (self.fdata,),
            Some({
                let dict = PyDict::new(py);
                // Clamp input range if requested
                match minmax {
                    Some((imin, imax)) => {
                        dict.set_item("in_range", (imin, imax))?;
                    }
                    None => (),
                }
                dict.set_item("out_range", (0.0, 1.0))?;
                dict
            }),
        )?;
        Ok(RescaledIntensityNiiImage::new(self))
    }

    pub(crate) fn get_slice(
        &self,
        py: Python<'a>,
        index: [isize; SECONDARY_DIMS],
    ) -> Result<&'a PyAny, ErrorTy> {
        debug_assert!(index.len() == self.dims.len() - 2);
        let slice = if self.dims[MAX_DIMS - 1] > 1 {
            self.fdata.get_item((
                // Slicing using : is not supported in PyO3 yet
                // https://github.com/PyO3/pyo3/issues/3000
                PySlice::new(py, 0, self.dims[0], 1),
                PySlice::new(py, 0, self.dims[1], 1),
                index[0],
                index[1],
            ))?
        } else {
            self.fdata.get_item((
                PySlice::new(py, 0, self.dims[0], 1),
                PySlice::new(py, 0, self.dims[1], 1),
                index[0],
            ))?
        };
        Ok(slice)
    }
}
