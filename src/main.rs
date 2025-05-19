mod xml_utils;
mod bitmap_utils;
mod array_utils;

use array_utils::reflect;
use xml_utils::get_base_tiles;
use bitmap_utils::load_bitmap;
use bitmap_utils::save_bitmap;
use array_utils::rotate;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = get_base_tiles("Circuit.xml")?;
    
    for tile in tiles {
        println!("Tile: {}, Variants: {:?}, Weight: {:?}", tile.name, tile.variants, tile.weight);
        println!("Reflecting variant 0 yields variant {}", (tile.reflect_v)(0));
    }

    let (pixels, width, height) = load_bitmap("connection.png");
    println!("Loaded image: {}x{}", width, height);

    let transformed = rotate(&pixels);
    let transformed = reflect(&transformed);

    save_bitmap("rotated.png", &transformed, width, height);
    println!("Saved rotated image to 'rotated.png'");

    Ok(())
}
