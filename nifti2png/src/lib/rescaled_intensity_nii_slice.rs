use pyo3::PyAny;

use crate::error_ty::ErrorTy;

pub(crate) struct RescaledIntensityNiiSlice<'a>(&'a PyAny);

fn gray2ubyte_rgb<'a>(
    slice: &'a PyAny,
    // from skimage import color
    color: &'a PyAny,
    // from skimage import img_as_ubyte
    img_as_ubyte: &'a PyAny,
) -> Result<&'a PyAny, ErrorTy> {
    let rgb = color.call_method("gray2rgb", (slice,), None)?;
    // this line silences a warning
    // https://github.com/zhixuhao/unet/issues/125
    let ubyte_rgb = img_as_ubyte.call((rgb,), None)?;
    Ok(ubyte_rgb)
}

fn save_ubyte_rgb_grayscale_slice(
    path: &PyAny,
    // ndarray of shape (N, M, 3) with dtype uint8.
    // It is a grayscale image with 3 channels
    // made from z-slice of the original multidimensional image
    ubyte_sz_rgb: &PyAny,
    // from skimage import io
    io: &PyAny,
    // from PIL import Image
    #[allow(non_snake_case)] Image: &PyAny,
    // from PIL import ImageOps
    #[allow(non_snake_case)] ImageOps: &PyAny,
) -> Result<(), ErrorTy> {
    io.call_method("imsave", (path, ubyte_sz_rgb), None)?;
    // TODO: use in-memory processing instead
    let color_image = Image.call_method("open", (path,), None)?;
    let rotated = color_image.call_method("rotate", (90,), None)?;
    let mirrored = ImageOps.call_method("mirror", (rotated,), None)?;
    mirrored.call_method("save", (path,), None)?;
    Ok(())
}

impl<'a> RescaledIntensityNiiSlice<'a> {
    pub(crate) fn new(nii_slice: &'a PyAny) -> Self {
        Self(nii_slice)
    }

    pub(crate) fn save(
        &self,
        path: &PyAny,
        io: &PyAny,
        color: &PyAny,
        img_as_ubyte: &PyAny,
        #[allow(non_snake_case)] Image: &PyAny,
        #[allow(non_snake_case)] ImageOps: &PyAny,
    ) -> Result<(), ErrorTy> {
        let ubyte_sz_rgb = gray2ubyte_rgb(self.0, color, img_as_ubyte)?;
        save_ubyte_rgb_grayscale_slice(path, ubyte_sz_rgb, io, Image, ImageOps)
    }
}
