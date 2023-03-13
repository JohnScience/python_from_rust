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
    #[error("{0}")]
    UncategorizedPyErr(#[from] PyErr),
}
