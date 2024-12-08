use std::error::Error;

use chameleon::formats::{png::Png, ppm::PpmSmall};

#[test]
fn test_zlib() -> Result<(), Box<dyn Error>> {
    // Create a png.
    let png = Png::from_path("./tests/samples/small_gradient.png")?;

    println!("{:?}", png.data);

    // Call rgb() which will push IDAT into zlib.
    let x = png.rgb();

    let ppm = PpmSmall::build(&x, 2, 2);

    ppm.write("./tests/output/output.ppm")?;

    Ok(())
}
