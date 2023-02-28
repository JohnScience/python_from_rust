use pyo3::prelude::*;

fn main() {
    println!(
        "Enter a path, e.g. {}",
        std::env::current_dir().unwrap().display()
    );
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    let path = buffer.trim_end();

    Python::with_gil(|py| {
        let os = py.import("os").unwrap();
        if os
            .getattr("path")
            .unwrap()
            .call_method("exists", (path,), None)
            .unwrap()
            .extract::<bool>()
            .unwrap()
        {
            println!("Path exists");
        } else {
            println!("Path does not exist");
        }
    });
}
