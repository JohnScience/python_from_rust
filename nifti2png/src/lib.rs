use pyo3::prelude::*;
use pyo3::types::PyUnicode;

pub fn convert(nii_files: &str, png_stub: Option<&str>, minmax: (u64, u64)) -> PyResult<()> {
    Python::with_gil(|py| {
        let os = py.import("os")?;
        let nib = py.import("nibabel")?;

        let png_stub = PyUnicode::new(py, png_stub.unwrap_or("slice"));

        for nii_file in os.call_method("listdir", (nii_files,), None)?.iter()? {
            let nii_file = nii_file?;
            let png_stub = os
                .getattr("path")?
                .call_method("join", (png_stub, nii_file), None)?;
            println!("png_stub: {:?}", png_stub);
            todo!();
        }
        Ok(())
    })
}
