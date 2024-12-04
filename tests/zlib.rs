use std::error::Error;

use chameleon::formats::png::Png;

#[test]
fn test_zlib() -> Result<(), Box<dyn Error>> {
    // Create a png.
    let png = Png::from_path("./tests/samples/minimal.png")?;

    // Call rgb() which will push IDAT into zlib.
    png.rgb();

    Ok(())
}
