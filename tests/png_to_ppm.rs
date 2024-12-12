use std::error::Error;

use chameleon::formats::{png::Png, ppm::Ppm};

#[test]
fn test_png() -> Result<(), Box<dyn Error>> {
    // Create a png.
    let png = Png::from_path("./tests/samples/gradient.png")?;

    // Call rgb() which will push IDAT into zlib.
    let x = png.rgb();

    let ppm = Ppm::build(&x, png.dimensions.0, png.dimensions.1);

    ppm.write("./tests/output/output.ppm")?;

    Ok(())
}
