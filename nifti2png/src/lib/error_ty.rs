use pyo3::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorTy {
    #[error("Missing standard Python library: {0}")]
    MissingStandardLibrary(PyErr),
    #[error("Missing third-party Python library: {0}")]
    MissingThirdPartyLibrary(PyErr),
    #[error("Missing component of third-party Python library: {0}")]
    MissingComponentOfThirdPartyLibrary(PyErr),
    #[error("os.listdir({1}) failed: {0}")]
    ListDirFailed(PyErr, String),
    #[error("std::fs::create_dir_all({1}) failed: {0}")]
    CreateDirAllFailed(std::io::Error, String),
    #[error("std::path::Path::try_exists({1}) failed: {0}")]
    TryExistsFailed(std::io::Error, String),
    #[error("image::open({1}) failed: {0}")]
    ImageOpenFailed(image::ImageError, String),
    #[error("std::env::temp_dir() returned a non-UTF-8 path")]
    TempDirNotUtf8,
    #[error("{0}")]
    UncategorizedPyErr(#[from] PyErr),
}
