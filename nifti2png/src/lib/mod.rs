use arrayvec::ArrayVec;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PySlice, PyUnicode};
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

use ErrorTy::*;

/// Expected number of dimensions in images.
const MAX_DIMS: usize = 4;

/// Iterator over pairs of (png_stub, nii_obj) for all nii files in nii_files
/// where png_stub is a path to a directory where the png files
/// for the NIFTI volume will be saved and nii_obj is a NIFTI object (with a header and data)
struct RelNiiFilesIter<'a> {
    nib: &'a PyModule,
    os: &'a PyModule,
    nii_files: &'a str,
    base_png_stub: &'a PyUnicode,
    listdir_iter: &'a PyIterator,
}

impl<'a> RelNiiFilesIter<'a> {
    fn new(
        nib: &'a PyModule,
        os: &'a PyModule,
        nii_files: &'a str,
        base_png_stub: &'a PyUnicode,
    ) -> Result<Self, ErrorTy> {
        let listdir_iter = match os.call_method("listdir", (nii_files,), None) {
            Ok(listdir_res) => listdir_res.iter()?,
            Err(e) => return Err(ListDirFailed(e, nii_files.to_string())),
        };
        Ok(Self {
            nib,
            os,
            nii_files,
            listdir_iter,
            base_png_stub,
        })
    }

    fn png_stub(&self, nii_file: &PyAny) -> Result<&'a PyAny, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L66
        let png_stub =
            self.os
                .getattr("path")?
                .call_method("join", (self.base_png_stub, nii_file), None)?;
        Ok(png_stub)
    }

    fn nii_obj(&self, nii_file: &PyAny) -> Result<&'a PyAny, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L75
        let nii_obj = self.nib.call_method(
            "load",
            ({
                // nii_files + "\\" + nii_file
                self.os
                    .getattr("path")?
                    .call_method("join", (self.nii_files, nii_file), None)?
            },),
            None,
        )?;
        Ok(nii_obj)
    }

    // TODO: improve naming
    fn item(
        &self,
        nii_file: PyResult<&'a PyAny>,
    ) -> Result<
        (
            // png_stub
            &'a PyAny,
            // nii_obj
            &'a PyAny,
        ),
        ErrorTy,
    > {
        let nii_file = nii_file?;

        let png_stub = self.png_stub(nii_file)?;
        let nii_obj = self.nii_obj(nii_file)?;

        Ok((png_stub, nii_obj))
    }
}

impl<'a> Iterator for RelNiiFilesIter<'a> {
    type Item = Result<
        (
            // png_stub
            &'a PyAny,
            // nii_obj
            &'a PyAny,
        ),
        ErrorTy,
    >;

    fn next(&mut self) -> Option<Self::Item> {
        self.listdir_iter
            .next()
            .map(|nii_file_res| self.item(nii_file_res))
    }
}

/// Loaded NIFTI image
struct NiiImage<'a> {
    fdata: &'a PyAny,
    dims: ArrayVec<isize, MAX_DIMS>,
}

impl<'a> NiiImage<'a> {
    fn rescale_intensity_to_unit_interval(&mut self, py: Python<'a>, exposure: &'a PyAny, minmax: Option<(u64,u64)>) -> Result<(), ErrorTy> {
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
                    },
                    None => (),
                }
                dict.set_item("out_range", (0.0, 1.0))?;
                dict
            }),
        )?;
        Ok(())
    }
}

impl<'a> core::ops::Index<[isize; 2]> for NiiImage<'a> {
    type Output = GrayscaleNiiSlice<'a>;

    fn index(&self, index: [isize; 2]) -> &Self::Output {
        if self.dims[MAX_DIMS-1] > 1 {
            
        }
    }
}

// Iterator over pairs of (png_stub, nii_image) for all nii files in nii_files
// where png_stub is a path to a directory where the png files for the NIFTI volume will be saved.
struct RelNiiImageIter<'a>(RelNiiFilesIter<'a>);

impl<'a> RelNiiImageIter<'a> {
    fn new(
        nib: &'a PyModule,
        os: &'a PyModule,
        nii_files: &'a str,
        base_png_stub: &'a PyUnicode,
    ) -> Result<Self, ErrorTy> {
        Ok(Self(RelNiiFilesIter::new(
            nib,
            os,
            nii_files,
            base_png_stub,
        )?))
    }

    fn nii_obj2nii_image(nii_obj: &'a PyAny) -> Result<NiiImage<'a>, ErrorTy> {
        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L78
        let hdr = nii_obj.getattr("header")?;

        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L81
        let nii_shape = hdr.call_method("get_data_shape", (), None)?;

        let mut dims = ArrayVec::new_const();

        for i in 0..=2 {
            dims.push(nii_shape.get_item(i)?.extract::<isize>()?);
        }

        dims.push(match nii_shape.len()? {
            3 => 1,
            4 => nii_shape.get_item(3)?.extract::<isize>()?,
            // TODO: specify which image is invalid
            _ => panic!("Unexpected dimensionality of the image (3 or 4 expected)"),
        });

        let fdata = nii_obj.call_method("get_fdata", (), None)?;

        Ok(NiiImage { fdata, dims })
    }

    fn nii_obj_res2nii_image_res(
        res: Result<(&'a PyAny, &'a PyAny), ErrorTy>,
    ) -> Result<(&'a PyAny, NiiImage<'a>), ErrorTy> {
        let (png_stub, nii_obj) = res?;
        let nii_image = Self::nii_obj2nii_image(nii_obj)?;
        Ok((png_stub, nii_image))
    }
}

impl<'a> Iterator for RelNiiImageIter<'a> {
    type Item = Result<(&'a PyAny, NiiImage<'a>), ErrorTy>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Self::nii_obj_res2nii_image_res)
    }
}

struct GrayscaleNiiSlice<'a>(&'a PyAny);

struct RelGrayscaleNiiSliceIter<'a>(RelNiiImageIter<'a>);

impl<'a> RelGrayscaleNiiSliceIter<'a> {
    fn new(
        nib: &'a PyModule,
        os: &'a PyModule,
        nii_files: &'a str,
        base_png_stub: &'a PyUnicode,
    ) -> Result<Self, ErrorTy> {
        Ok(Self(RelNiiImageIter::new(
            nib,
            os,
            nii_files,
            base_png_stub,
        )?))
    }
}

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
pub fn convert(
    nii_files: &str,
    png_stub: Option<&str>,
    minmax: Option<(u64, u64)>,
) -> Result<(), ErrorTy> {
    Python::with_gil(|py| {
        let os = py.import("os").map_err(MissingStandardLibrary)?;
        let nib = py.import("nibabel").map_err(MissingThirdPartyLibrary)?;
        let np = py.import("numpy").map_err(MissingThirdPartyLibrary)?;
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
        let png_stub = PyUnicode::new(py, png_stub.unwrap_or("slice"));

        for res in RelNiiImageIter::new(nib, os, nii_files, png_stub)? {
            let (png_stub, nii_image) = res?;

            println!("\tMatrix size: ({:?}", nii_image.dims);

            nii_image.rescale_intensity_to_unit_interval(py, exposure, minmax);

            for t in 0..=nt {
                let png_dir = png_stub;

                println!("\tVolume {t} -> {png_dir}");

                // Create png_dir if necessary
                if !(os
                    .getattr("path")?
                    .call_method("exists", (png_dir,), None)?
                    .extract::<bool>()?)
                {
                    os.call_method("makedirs", (png_dir,), None)?;
                }

                // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L119-L122

                // Current volume
                let st = if nt > 1 {
                    s.get_item((
                        // Slicing using : is not supported in PyO3 yet
                        // https://github.com/PyO3/pyo3/issues/3000
                        PySlice::new(py, 0, nx, 1),
                        PySlice::new(py, 0, ny, 1),
                        PySlice::new(py, 0, nz, 1),
                        t,
                    ))?
                } else {
                    s.get_item((
                        PySlice::new(py, 0, nx, 1),
                        PySlice::new(py, 0, ny, 1),
                        PySlice::new(py, 0, nz, 1),
                    ))?
                };

                for z in 0..nz {
                    // PNG filename
                    let png_path = os.getattr("path")?.call_method(
                        "join",
                        (png_dir, format!("{z:04}.png")),
                        None,
                    )?;

                    let grayscale_slice =
                        st.get_item((PySlice::new(py, 0, nx, 1), PySlice::new(py, 0, ny, 1), z))?;

                    // Write single byte image slice to jpg file
                    let ubyte_sz_rgb = gray2ubyte_rgb(grayscale_slice, color, img_as_ubyte)?;
                    save_ubyte_rgb_grayscale_slice(png_path, ubyte_sz_rgb, io, Image, ImageOps)?;
                }
            }
        }

        println!("Done");

        Ok(())
    })
}
