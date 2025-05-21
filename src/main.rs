use simple_tiled::SimpleTiledModel;

mod simple_tiled;
mod bitmap_utils;
mod array_utils;


fn main() -> Result<(), Box<dyn std::error::Error>> {
/* let (t, tilesize, weights, tiles, tilenames, action, first_occurrence, propagator) = initialize_simple_tiled_model("Circuit.xml")?;

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


    println!("Contenuto del propagator:");
    for direction_index in 0..propagator.len() {
        println!("Direzione {}", direction_index);
        let direction = &propagator[direction_index];
        for tile_id1 in 0..direction.len() {
            print!("  Per il tile {tile_id1}: ");
            let tile = &direction[tile_id1];
            for tile_id2 in 0..tile.len() {
                print!("{}, ", propagator[direction_index][tile_id1][tile_id2]);
            }
            println!();
        }
    }*/
    
    let mut model: SimpleTiledModel = SimpleTiledModel::new("Circuit.xml", 50)?;

    let success = model.run(10, [42; 32]);

    if success {
       println!("Non è esploso niente!"); 
    }

    Ok(())
}
