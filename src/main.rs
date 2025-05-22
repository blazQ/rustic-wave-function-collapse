use simple_tiled::SimpleTiledModel;
use clap::Parser;
use rand::{Rng};
mod simple_tiled;
mod bitmap_utils;
mod array_utils;

/// Parametri da linea di comando
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Tileset name, choose between XML files
    #[arg(short, long, default_value = "Summer")]
    tileset: String,

    /// Grid dimension (doesn't correspond to image size, depends on tile size)
    #[arg(short, long, default_value_t = 10)]
    size: usize,

    /// Iterations (-1 means go until you reach an end state)
    #[arg(short, long, default_value_t = -1)]
    limit: isize,

    /// Seed (if not specified, randomly generated)
    #[arg(long)]
    seed: Option<u64>,

    /// Optionally show text output
    #[arg(long, default_value_t = false)]
    text: bool,

    /// Name of the .png output
    #[arg(short, long, default_value = "output.png")]
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml_path = format!("./tilesets/{}.xml", args.tileset);
    let mut model = SimpleTiledModel::new(&xml_path, args.size)?;

    // Seed generation
    let seed_arr: [u8; 32] = if let Some(seed) = args.seed {
        let mut arr = [0u8; 32];
        arr[..8].copy_from_slice(&seed.to_le_bytes());
        arr
    } else {
        let mut arr = [0u8; 32];
        rand::rng().fill(&mut arr);
        arr
    };

    let success = model.run(args.limit, seed_arr);

    if success {
        println!("Success!:");
        println!("-------------------");
        model.save(&args.output);
        if args.text {
            println!("{}", model.text_output());
        }
    } else {
        println!("CONTRADICTION");
    }
    Ok(())
}