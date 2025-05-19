use roxmltree::Document;
use std::fs;

#[derive(Debug, Clone)]
pub struct Tile {
    pub name: String,
    pub variants: u8,
    pub rotate_v: fn(u8) -> u8,
    pub reflect_v: fn(u8) -> u8,
    pub weight: f32,
}

fn get_variant_functions(symmetry: &str) -> (u8, fn(u8) -> u8, fn(u8) -> u8) {
    match symmetry {
        "I" => (2, |x| 1 - x, |x| x),
        "L" => (4, |x| (x + 1) % 4, |x| if x % 2 == 0 {x + 1} else {x - 1}),
        "//" => (2, |x| 1 - x, |x| 1 - x),
        "F" => (8, |x| if x < 4 {(x + 1) % 4} else {4 + (x - 1) % 4}, |x| if x < 4 {x + 4} else {x - 4}),
        "T" => (4, |x| x + 1 % 4, |x| if x % 2 == 0 {x} else { 4 - x}),
        _ => (1, |x| x, |x| x),
    }
}

pub fn get_base_tiles(xml_path: &str) -> Result<Vec<Tile>, Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(xml_path)?;
    let doc = Document::parse(&xml_content)?;
    let tiles_tag = doc.descendants()
        .find(|n| n.has_tag_name("tiles"))
        .ok_or_else(|| String::from("Tag <tiles> non trovato nel documento"))?;
    
    let tiles: Vec<Tile> = tiles_tag.children()
        .filter(|n| n.has_tag_name("tile"))
        .map(|n| {
            let symmetry = n.attribute("symmetry").unwrap_or("");
            let (variants, rotate_v, reflect_v) = get_variant_functions(symmetry);
            Tile {
                name: n.attribute("name").unwrap_or("").to_string(),
                variants,
                rotate_v,
                reflect_v,
                weight: n.attribute("weight")
                    .and_then(|w| w.parse::<f32>().ok())
                    .unwrap_or(1.0),
                }
        })
        .collect();
    
    Ok(tiles)
}