use pyo3::prelude::*;
use thiserror::Error;

mod nifti_image;
mod python_deps;
mod rescaled_intensity_nifti_image;

pub use python_deps::PythonDeps;
pub use rescaled_intensity_nifti_image::RescaledIntensityNiftiImage;

pub const MAX_DIMS: usize = 4;
pub const PRIMARY_DIMS: usize = 2;
pub const SECONDARY_DIMS: usize = MAX_DIMS - PRIMARY_DIMS;

#[derive(Debug, Error)]
pub enum ErrorTy {
    #[error("Failed to load Nifti object from {1}: {0}")]
    FailedToLoadNiftiObj(PyErr, String),
    #[error("The NIFTI image {1} has an unsupported dimensionality: {0} (expected 3 or 4)")]
    UnsupportedDimensionality(usize, String),
    #[error("image::open({1}) failed: {0}")]
    ImageOpenFailed(image::ImageError, String),
    #[error("Missing a third-party Python library: {0}")]
    MissingThirdPartyLibrary(PyErr),
    #[error("Missing a component of a third-party Python library: {0}")]
    MissingComponentOfThirdPartyLibrary(PyErr),
    #[error("A path to the temporary directory is not UTF-8.")]
    TempDirIsNotUtf8,
    #[error("Uncategorised Python error: {0}")]
    UncategorisedPyError(#[from] PyErr),
}
