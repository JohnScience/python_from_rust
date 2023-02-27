use pyo3::prelude::*;
use pyo3::types::PyUnicode;

fn main() {
    println!("Enter a path, e.g. {}", std::env::current_dir().unwrap().display());
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    Python::with_gil(|py| {
        let os = py.import("os").unwrap();
        let path = PyUnicode::new(py, buffer.trim_end());

        for entry in os.call_method("listdir", (path,), None).unwrap().iter().unwrap() {
            println!("{}", entry.unwrap());
        }
    })
}
