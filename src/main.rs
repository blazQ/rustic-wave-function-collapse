use simple_tiled::SimpleTiledModel;

mod simple_tiled;
mod bitmap_utils;
mod array_utils;


fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let mut model: SimpleTiledModel = SimpleTiledModel::new("Castle.xml", 100)?;

    let success = model.run(-1, [42; 32]);

    if success {
       println!("Non è esploso niente!");
       println!("Ecco il risultato:");
       println!("-------------------");
       println!("{}", model.text_output());
       model.save("output.png");
    }

    Ok(())
}
