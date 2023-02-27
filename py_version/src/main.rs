use pyo3::prelude::*;

fn main() {
    Python::with_gil(|py| {
        let sys = py.import("sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        println!("This is Python {}", version);
    });
}
