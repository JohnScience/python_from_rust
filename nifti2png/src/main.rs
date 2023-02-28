use nifti2png::convert;

fn main() {
    println!(
        "Enter a path to a directory with NIFTI files, e.g. {example_asset}",
        example_asset = {
            let mut buf = std::env::current_dir().unwrap();
            buf.push("assets");
            buf
        }
        .display()
    );
    let mut nii_files = String::new();
    std::io::stdin().read_line(&mut nii_files).unwrap();
    let nii_files = nii_files.trim_end();

    println!("Enter the `png_stub`:");
    let mut png_stub = String::new();
    std::io::stdin().read_line(&mut png_stub).unwrap();
    let png_stub = match png_stub.trim_end() {
        "" => None,
        png_stub => Some(png_stub),
    };

    println!("Enter the `minmax`:");
    let mut minmax = String::new();
    std::io::stdin().read_line(&mut minmax).unwrap();
    let min_max = match minmax
        .trim_end()
        .split_whitespace()
        .map(|s| s.parse::<u64>().unwrap())
        .collect::<Vec<_>>()[..]
    {
        [min, max] => (min, max),
        _ => panic!("Invalid input"),
    };

    convert(nii_files, png_stub, min_max).unwrap();
}
