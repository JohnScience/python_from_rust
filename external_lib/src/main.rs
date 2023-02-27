use pyo3::prelude::*;
use pyo3::types::PyUnicode;

fn main() {
    let example_asset = {
        let mut buf = std::env::current_dir().unwrap();
        buf.pop();
        buf.push("assets");
        buf.push("avg152T1_LR_nifti.nii.gz");
        buf
    };
    println!("Enter a path, e.g. {}", example_asset.display());
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    Python::with_gil(|py| {
        let nib = py.import("nibabel").unwrap();
        let path = PyUnicode::new(py, buffer.trim_end());

        let nii_file = nib.call_method("load", (path,), None).unwrap();
        let fdata = nii_file.call_method0("get_fdata").unwrap();
        let shape = fdata.getattr("shape").unwrap();
        println!("Shape: {:?}", shape);
    })
}
