use nifti_slice::{PythonDeps, RescaledIntensityNiftiImage};
use pyo3::prelude::*;

pub fn main() {
    Python::with_gil(|py| {
        println!(
            "Enter a path to a NIFTI file, e.g. {example_asset}",
            example_asset = {
                let mut buf = std::env::current_dir().unwrap();
                match &buf {
                    p if p.ends_with("python_from_rust") => {
                        buf.push("assets");
                        buf.push("avg152T1_LR_nifti.nii.gz");
                    }
                    p if p.ends_with("nifti_slice") => {
                        buf.pop();
                        buf.push("assets");
                        buf.push("avg152T1_LR_nifti.nii.gz");
                    }
                    _ => panic!("The current directory is neither a crate root nor a workplace root"),
                };
                buf
            }
            .display()
        );
        let mut nii_files = String::new();
        std::io::stdin().read_line(&mut nii_files).unwrap();
        let nii_file = nii_files.trim_end();
    
        println!("Enter the `minmax`:");
        let mut minmax = String::new();
        std::io::stdin().read_line(&mut minmax).unwrap();
        let min_max = match minmax
            .trim_end()
            .split_whitespace()
            .map(|s| s.parse::<u64>())
            .collect::<Vec<_>>()[..]
        {
            [] => None,
            [Ok(min), Ok(max)] => Some((min, max)),
            _ => panic!("Invalid input"),
        };

        let py_deps = PythonDeps::new(py).unwrap();

        let nifti = RescaledIntensityNiftiImage::new(&py_deps, nii_file, min_max).unwrap();
        let [s, t] = nifti.secondary_dims();
        
        loop {
            let mut buf = String::new();
            println!("Enter the 2D index for [0..{s}, 0..{t}] secondary dimension or `exit`");
            std::io::stdin().read_line(&mut buf).unwrap();
            match buf {
                buf if buf.starts_with("exit") => break,
                buf => {
                    let idx = match buf.split_whitespace()
                    .map(str::parse::<isize>)
                    .collect::<Vec<_>>()[..] {
                        [Ok(x), Ok(y)] => [x, y],
                        _ => panic!("Invalid input"),
                    };
                    let png = nifti.slice_as_raw_rgba(&py_deps, idx).unwrap();
                    dbg!(png);
                }
            }
        };
    });
}