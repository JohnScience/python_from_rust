use pyo3::prelude::*;
use pyo3::types::PyUnicode;
use crate::ErrorTy::{*, self};

pub(crate) struct Tempfile<'a> {
    pub(crate) py_str: &'a pyo3::types::PyString,
    pub(crate) rust_path_buf: std::path::PathBuf,
}

#[allow(non_snake_case)]
pub struct PythonDeps<'a> {
    pub(crate) py: Python<'a>,
    // import nibabel as nib
    pub(crate) nib: &'a PyModule,
    // from skimage import exposure
    pub(crate) exposure: &'a PyAny,
    // from skimage import img_as_ubyte
    pub(crate) img_as_ubyte: &'a PyAny,
    // from skimage import color
    pub(crate) color: &'a PyAny,
    // from skimage import io
    pub(crate) io: &'a PyAny,
    // from PIL import Image
    pub(crate) Image: &'a PyAny,
    // from PIL import ImageOps
    pub(crate) ImageOps: &'a PyAny,
    // the dual representation of a temporary file path
    pub(crate) tempfile: Tempfile<'a>,
}

impl<'a> PythonDeps<'a> {
    pub fn new(py: Python<'a>) -> Result<Self, ErrorTy> {
        let nib = py.import("nibabel").map_err(MissingThirdPartyLibrary)?;
        let (io, color, exposure, img_as_ubyte) = {
            let skimage = py.import("skimage").map_err(MissingThirdPartyLibrary)?;
            (
                skimage
                    .getattr("io")
                    .map_err(MissingComponentOfThirdPartyLibrary)?,
                skimage
                    .getattr("color")
                    .map_err(MissingComponentOfThirdPartyLibrary)?,
                skimage
                    .getattr("exposure")
                    .map_err(MissingComponentOfThirdPartyLibrary)?,
                skimage
                    .getattr("img_as_ubyte")
                    .map_err(MissingComponentOfThirdPartyLibrary)?,
            )
        };

        // https://github.com/PyO3/pyo3/discussions/3001

        #[allow(non_snake_case)]
        let Image = py.import("PIL.Image").map_err(MissingThirdPartyLibrary)?;
        #[allow(non_snake_case)]
        let ImageOps = py
            .import("PIL.ImageOps")
            .map_err(MissingThirdPartyLibrary)?;

        let tempfile = std::env::temp_dir().join("tempfile.png");
        let py_str = tempfile.to_str().ok_or(TempDirIsNotUtf8)?;

        Ok(PythonDeps {
            py,
            nib,
            io,
            color,
            exposure,
            img_as_ubyte,
            Image,
            ImageOps,
            tempfile: Tempfile {
                py_str: PyUnicode::new(py, py_str),
                rust_path_buf: tempfile,
            },
        })
    }
}
