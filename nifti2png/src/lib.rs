use pyo3::types::{PyDict, PyFloat, PySlice, PyTuple, PyUnicode};
use pyo3::prelude::*;

/// Notable differences from the original Python code:
///
/// - The original code uses concatenation with `\\` to construct paths.
/// - The original code does not format numbers with leading zeros.
pub fn convert(
    nii_files: &str,
    png_stub: Option<&str>,
    minmax: Option<(u64, u64)>,
) -> PyResult<()> {
    Python::with_gil(|py| {
        let os = py.import("os")?;
        let nib = py.import("nibabel")?;
        let np = py.import("numpy")?;
        let (io, color, exposure) = {
            let skimage = py.import("skimage")?;
            (
                skimage.getattr("io")?,
                skimage.getattr("color")?,
                skimage.getattr("exposure")?,
            )
        };

        // https://github.com/PyO3/pyo3/discussions/3001

        #[allow(non_snake_case)]
        let Image = py.import("PIL.Image")?;
        #[allow(non_snake_case)]
        let ImageOps = py.import("PIL.ImageOps")?;

        // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L61-L64
        let png_stub = PyUnicode::new(py, png_stub.unwrap_or("slice"));

        for nii_file in os.call_method("listdir", (nii_files,), None)?.iter()? {
            let nii_file = nii_file?;
            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L66
            let png_stub = os
                .getattr("path")?
                .call_method("join", (png_stub, nii_file), None)?;
            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L69
            let (nii_stub, fext) = {
                // os.path.splitext(nii_files + "\\" + nii_file)
                let tuple = os
                    .getattr("path")?
                    .call_method(
                        "splitext",
                        ({
                            // nii_files + "\\" + nii_file in https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L69
                            os.getattr("path")?
                                .call_method("join", (nii_files, nii_file), None)?
                        },),
                        None,
                    )?
                    .downcast::<PyTuple>()?;
                match (tuple.get_item(0), tuple.get_item(1)) {
                    (Ok(nii_stub), Ok(fext)) => (nii_stub, fext),
                    _ => panic!("Invalid input"),
                }
            };
            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L70-L71
            let _nii_stub = if fext.eq(".gz")? {
                os.getattr("path")?
                    .call_method("splitext", (nii_stub,), None)?
                    .get_item(0)?
            } else {
                nii_stub
            };

            println!("Opening NIFTI-1 volume");

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L75
            let nii_obj = nib.call_method(
                "load",
                ({
                    // nii_files + "\\" + nii_file
                    os.getattr("path")?
                        .call_method("join", (nii_files, nii_file), None)?
                },),
                None,
            )?;

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L78
            let hdr = nii_obj.getattr("header")?;

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L81
            let nii_shape = hdr.call_method("get_data_shape", (), None)?;

            let [nx, ny, nz] = [
                nii_shape.get_item(0)?.extract::<isize>()?,
                nii_shape.get_item(1)?.extract::<isize>()?,
                nii_shape.get_item(2)?.extract::<isize>()?,
            ];

            let nt = match nii_shape.len()? {
                3 => 1,
                4 => nii_shape.get_item(3)?.extract::<isize>()?,
                _ => panic!("Invalid input"),
            };

            println!("Loading voxel data");

            let s = nii_obj.call_method("get_fdata", (), None)?;

            println!("\tMatrix size: ({nx}, {ny}, {nz}, {nt})");

            // Clamp input range if requested
            let (imin, imax) = match minmax {
                Some((min, max)) => (PyFloat::new(py, min as f64), PyFloat::new(py, max as f64)),
                None => (
                    np.call_method("min", (s,), None)?.downcast()?,
                    np.call_method("max", (s,), None)?.downcast()?,
                ),
            };

            // Rescale to 0..255 uint8
            let s = exposure.call_method(
                "rescale_intensity",
                (s,),
                Some({
                    let dict = PyDict::new(py);
                    dict.set_item("in_range", (imin, imax))?;
                    dict.set_item("out_range", (0.0, 1.0))?;
                    dict
                }),
            )?;

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

                    // Write single byte image slice to jpg file
                    let sz_rgb = color.call_method(
                        "gray2rgb",
                        (st.get_item((
                            PySlice::new(py, 0, nx, 1),
                            PySlice::new(py, 0, ny, 1),
                            z,
                        ))?,),
                        None,
                    )?;

                    io.call_method(
                        "imsave",
                        (png_path, sz_rgb),
                        None,
                    )?;

                    // TODO: use in-memory processing instead
                    let color_image = Image.call_method("open", (png_path,), None)?;
                    let rotated = color_image.call_method("rotate", (90,), None)?;
                    let mirrored = ImageOps.call_method("mirror", (rotated,), None)?;
                    mirrored.call_method("save", (png_path,), None)?;
                }
            }
        }

        println!("Done");

        Ok(())
    })
}
