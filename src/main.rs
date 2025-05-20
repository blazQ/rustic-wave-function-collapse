use std::collections::HashMap;

use bitmap_utils::save_bitmap;
use simple_tiled::initialize_simple_tiled_model;

mod simple_tiled;
mod bitmap_utils;
mod array_utils;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (t, tilesize, weights, tiles, tilenames, action, first_occurrence) = initialize_simple_tiled_model("Castle.xml")?;

    println!("Tiles: {}", t);

    for tilename in first_occurrence.keys(){
        println!("Name: {}, First_occurence: {}", tilename, first_occurrence[tilename]);
    }

    for weight in weights {
        println!("Weights: {}", weight);
    }

    for (index, tile) in tiles.iter().enumerate() {
        save_bitmap(format!("{index}.png"), tile, tilesize, tilesize);
    }

    for (index, tile) in action.iter().enumerate(){
        println!("Tile no. {}", index);
        println!("-------------------");
        for (index, variant) in tile.iter().enumerate() {
            println!("Transform {index} yields {variant}");
        }
    }

    Ok(())
}
