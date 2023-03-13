use image::RgbaImage;
use pyo3::PyAny;

use crate::{nifti_image::NiftiImage, ErrorTy, PythonDeps, SECONDARY_DIMS};

fn gray2ubyte_rgb<'a>(py_deps: &PythonDeps<'a>, slice: &'a PyAny) -> Result<&'a PyAny, ErrorTy> {
    let rgb = py_deps.color.call_method("gray2rgb", (slice,), None)?;
    // this line silences a warning
    // https://github.com/zhixuhao/unet/issues/125
    let ubyte_rgb = py_deps.img_as_ubyte.call((rgb,), None)?;
    Ok(ubyte_rgb)
}

fn save_ubyte_rgb_grayscale_slice<'a>(
    py_deps: &PythonDeps<'a>,
    path: &PyAny,
    // ndarray of shape (N, M, 3) with dtype uint8.
    // It is a grayscale image with 3 channels
    // made from z-slice of the original multidimensional image
    ubyte_sz_rgb: &PyAny,
) -> Result<(), ErrorTy> {
    py_deps
        .io
        .call_method("imsave", (path, ubyte_sz_rgb), None)?;
    // TODO: use in-memory processing instead
    let color_image = py_deps.Image.call_method("open", (path,), None)?;
    let rotated = color_image.call_method("rotate", (90,), None)?;
    let mirrored = py_deps.ImageOps.call_method("mirror", (rotated,), None)?;
    mirrored.call_method("save", (path,), None)?;
    Ok(())
}

pub struct RescaledIntensityNiftiImage<'a>(pub(crate) NiftiImage<'a>);

impl<'a> RescaledIntensityNiftiImage<'a> {
    pub fn new(py_deps: &PythonDeps<'a>, path: &str, minmax: Option<(u64, u64)>) -> Result<Self, ErrorTy> {
        let nii = NiftiImage::open(&py_deps, path)?;
        nii.rescale_intensity_to_unit_interval(&py_deps, minmax)
    }

    pub fn primary_dims(&self) -> [isize; SECONDARY_DIMS] {
        self.0.primary_dims()
    }

    pub fn secondary_dims(&self) -> [isize; SECONDARY_DIMS] {
        self.0.secondary_dims()
    }

    fn slice(
        &self,
        py_deps: &PythonDeps<'a>,
        index: [isize; SECONDARY_DIMS],
    ) -> Result<&'a PyAny, ErrorTy> {
        self.0.slice(py_deps, index)
    }

    fn save_slice(
        &self,
        py_deps: &PythonDeps<'a>,
        path: &PyAny,
        index: [isize; SECONDARY_DIMS],
    ) -> Result<(), ErrorTy> {
        let slice = self.slice(py_deps, index)?;
        let ubyte_sz_rgb = gray2ubyte_rgb(py_deps, slice)?;
        save_ubyte_rgb_grayscale_slice(py_deps, path, ubyte_sz_rgb)
    }

    fn slice_as_rgba(
        &self,
        py_deps: &PythonDeps<'a>,
        idx: [isize; SECONDARY_DIMS],
    ) -> Result<RgbaImage, ErrorTy> {
        let path = &py_deps.tempfile.rust_path_buf;
        self.save_slice(py_deps, py_deps.tempfile.py_str, idx)?;
        let img = image::open(path)
            .map_err(|e| ErrorTy::ImageOpenFailed(e, path.to_string_lossy().to_string()))?;
        Ok(img.to_rgba8())
    }

    pub fn slice_as_raw_rgba(
        &self,
        py_deps: &PythonDeps<'a>,
        idx: [isize; SECONDARY_DIMS],
    ) -> Result<Vec<u8>, ErrorTy> {
        let img = self.slice_as_rgba(py_deps, idx)?;
        Ok(img.into_raw())
    }
}
