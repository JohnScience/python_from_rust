use arrayvec::ArrayVec;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySlice, PyUnicode};

use crate::{ErrorTy, PythonDeps, RescaledIntensityNiftiImage, MAX_DIMS, SECONDARY_DIMS, PRIMARY_DIMS};

pub(crate) struct NiftiImage<'a> {
    pub(crate) fdata: &'a PyAny,
    pub(crate) dims: ArrayVec<isize, MAX_DIMS>,
}

impl<'a> NiftiImage<'a> {
    pub(crate) fn open(py_deps: &PythonDeps<'a>, path: &str) -> Result<Self, ErrorTy> {
        let py_path = PyUnicode::new(py_deps.py, path);
        let nii_obj = py_deps
            .nib
            .call_method1("load", (py_path,))
            .map_err(|e| ErrorTy::FailedToLoadNiftiObj(e, path.to_string()))?;
        let fdata = nii_obj.call_method0("get_fdata")?;
        let hdr = nii_obj.getattr("header")?;
        let nii_shape = hdr.call_method0("get_data_shape")?;
        let mut dims = ArrayVec::new();
        for i in 0..=2 {
            dims.push(nii_shape.get_item(i)?.extract::<isize>()?);
        }
        dims.push(match nii_shape.len()? {
            3 => 1,
            4 => nii_shape.get_item(3)?.extract::<isize>()?,
            len => return Err(ErrorTy::UnsupportedDimensionality(len, path.to_string())),
        });

        Ok(NiftiImage { fdata, dims })
    }

    pub(crate) fn primary_dims(&self) -> [isize; PRIMARY_DIMS] {
        std::array::from_fn(|i| self.dims[i])
    }

    pub(crate) fn secondary_dims(&self) -> [isize; SECONDARY_DIMS] {
        std::array::from_fn(|i| self.dims[i + PRIMARY_DIMS])
    }

    pub(crate) fn rescale_intensity_to_unit_interval(
        mut self,
        py_deps: &PythonDeps<'a>,
        minmax: Option<(u64, u64)>,
    ) -> Result<RescaledIntensityNiftiImage<'a>, ErrorTy> {
        // Rescale to 0..255 uint8
        self.fdata = py_deps.exposure.call_method(
            "rescale_intensity",
            (self.fdata,),
            Some({
                let dict = PyDict::new(py_deps.py);
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
        Ok(RescaledIntensityNiftiImage(self))
    }

    pub(crate) fn slice(
        &self,
        py_deps: &PythonDeps<'a>,
        index: [isize; SECONDARY_DIMS],
    ) -> Result<&'a PyAny, ErrorTy> {
        debug_assert!(index.len() == self.dims.len() - 2);
        let slice = if self.dims[MAX_DIMS - 1] > 1 {
            self.fdata.get_item((
                // Slicing using : is not supported in PyO3 yet
                // https://github.com/PyO3/pyo3/issues/3000
                PySlice::new(py_deps.py, 0, self.dims[0], 1),
                PySlice::new(py_deps.py, 0, self.dims[1], 1),
                index[0],
                index[1],
            ))?
        } else {
            self.fdata.get_item((
                PySlice::new(py_deps.py, 0, self.dims[0], 1),
                PySlice::new(py_deps.py, 0, self.dims[1], 1),
                index[0],
            ))?
        };
        Ok(slice)
    }
}
