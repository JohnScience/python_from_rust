use std::path::PathBuf;

use pyo3::prelude::*;

mod error_ty;
use error_ty::ErrorTy::{self, *};
mod nii_image;
mod rel_nii_files_iter;
mod rel_nii_images_iter;
mod rescaled_intensity_nii_image;
mod rescaled_intensity_nii_slice;
pub mod target_path;
use rel_nii_images_iter::RelNiiImagesIter;

use crate::{
    nii_image::NiiImage, rescaled_intensity_nii_image::RescaledIntensityNiiImage,
    target_path::TargetImageDir,
};

/// Expected number of dimensions in images.
const MAX_DIMS: usize = 4;
/// All dimensions except for the first two.
/// This value is used for iteration over 2D-slices.
const SECONDARY_DIMS: usize = MAX_DIMS - 2;

/// Notable differences from the original Python code:
///
/// - The original code uses concatenation with `\\` to construct paths.
/// - The original code does not format numbers with leading zeros.
/// - The original code ignored the warning about lossy conversion:
/// Lossy conversion from float64 to uint8. Range \[0, 1\]. Convert image to uint8 prior to saving to suppress this warning.
/// Learn more about the warning [here](https://github.com/zhixuhao/unet/issues/125).
/// - The original code contains the
/// [dead code with `nii_stub`](https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L69-L71)
/// - The original code used np.min() and np.max() to calculate min and max values of the image
/// to later then pass them as `in_range` optional parameter to `skimage.exposure.rescale_intensity`.
/// - The original code iterated over the 4th dimension of the image but only the last 3D slice was used.
pub fn convert(
    nii_files: &str,
    png_stub: Option<&str>,
    minmax: Option<(u64, u64)>,
) -> Result<(), ErrorTy> {
    Python::with_gil(|py| {
        let os = py.import("os").map_err(MissingStandardLibrary)?;
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

        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L61-L64
        let png_stub = PathBuf::from(png_stub.unwrap_or("slice"));

        for res in RelNiiImagesIter::new(nib, os, nii_files, png_stub)? {
            let (png_stub, nii_image): (TargetImageDir, NiiImage) = res?;
            TargetImageDir::ensure_exists(&png_stub, os)?;

            println!("\tMatrix size: ({:?}", nii_image.dims);

            let nii_image: RescaledIntensityNiiImage =
                nii_image.rescale_intensity_to_unit_interval(py, exposure, minmax)?;

            for t in 0..nii_image.dim(MAX_DIMS - 1) {
                println!("\tVolume {t} -> {}", png_stub.path.display());

                for z in 0..nii_image.dim(MAX_DIMS - 2) {
                    // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L119-L122
                    // Current volume
                    let nii_slice = nii_image.get_slice(py, [z, t])?;

                    // PNG filename
                    let png_path = os.getattr("path")?.call_method(
                        "join",
                        (&png_stub.path, format!("{z:04}.png")),
                        None,
                    )?;

                    nii_slice.save(png_path, io, color, img_as_ubyte, &Image, &ImageOps)?;
                }
            }
        }

        println!("Done");

        Ok(())
    })
}
