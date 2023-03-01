use pyo3::prelude::*;
use pyo3::types::{PyUnicode, PyTuple};

pub fn convert(nii_files: &str, png_stub: Option<&str>, minmax: (u64, u64)) -> PyResult<()> {
    Python::with_gil(|py| {
        let os = py.import("os")?;
        let nib = py.import("nibabel")?;

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
                let tuple = os.getattr("path")?.call_method(
                    "splittext",
                    ({
                        // nii_files + "\\" + nii_file in https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L69
                        os.getattr("path")?
                            .call_method("join", (nii_files, nii_file), None)?
                    },),
                    None,
                )?.downcast::<PyTuple>()?;
                match (tuple.get_item(0), tuple.get_item(1)) {
                    (Ok(nii_stub), Ok(fext)) => (nii_stub, fext),
                    _ => panic!("Invalid input"),
                }
            };
            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L70-L71
            let nii_stub = if fext.eq(".gz")? {
                os.getattr("path")?
                    .call_method("splitext", (nii_stub,), None)?
                    .get_item(0)?
            } else { nii_stub };

            println!("Opening NIFTI-1 volume");

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L75
            let nii_obj = nib.call_method("load", ({
                // nii_files + "\\" + nii_file
                os.getattr("path")?
                    .call_method("join", (nii_files, nii_file), None)?
            },), None)?;

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L78
            let hdr = nii_obj.getattr("header")?;

            // https://github.com/korepanov/repalungs/blob/b8c3f62f3015ed89fc360a2a7166a29b56d293f4/back/converter/converter.py#L81
            let nii_shape = hdr.call_method("get_data_shape", (), None)?;
            
            let [nx,ny,nz] = [nii_shape.get_item(0)?,
                nii_shape.get_item(1)?,
                nii_shape.get_item(2)?];

            let nt = if nii_shape.len()? == 3 {
                1
            } else {
                nii_shape.get_item(3)?.extract::<usize>()?
            };

            dbg!(nii_shape);
            todo!();
        }
        Ok(())
    })
}
