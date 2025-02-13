use std::fs;

use chameleon::formats::{png::Png, ppm::Ppm};
/// Tests against every image in the png suite data set: www.shaick.com/pngsuite/
#[test]
pub fn png_suite() {
    let files = fs::read_dir("./tests/samples/").unwrap();

    for result in files {
        let file = result.unwrap();
        println!("Decoding {:?}", file.file_name());

        let png = match Png::from_path(file.path()) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("Failed to generate PNG struct from {:?}", file.file_name());
                std::process::exit(1);
            }
        };

        println!("Converting {:?} to RGB", file.file_name());
        let rgb = png.rgb();

        println!("Converting to PPM");
        let ppm = Ppm::build(&rgb, png.dimensions.0, png.dimensions.1);

        println!("Writing PPM file");
        match ppm.write(format!("./tests/output/{:?}.ppm", file.file_name())) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Failed to write PPM to output.");
                std::process::exit(1);
            }
        }
    }
}
