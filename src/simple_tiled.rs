use roxmltree::{Document};
use std::{collections::HashMap, fs};

use crate::{array_utils::{reflect, rotate}, bitmap_utils};

pub fn get_cardinality_a_b_on_symmetry(symmetry: &str) -> (usize, fn(usize) -> usize, fn(usize) -> usize){
    match symmetry {
        "L" => {(4, |x| (x+1)%4, |x| if x % 2 == 0 {x + 1} else { x - 1} )},
        "T" => {(4, |x| (x+1)%4, |x| if x % 2 == 0 {x} else { 4 - x} )},
        "I" => {(2, |x| 1 - x, |x| x)},
        "\\" => {(2, |x| 1 - x, |x| 1 - x )},
        "F" => {(8, |x| if x < 4 {(x + 1) % 4} else { 4 + (x - 1) % 4}, |x| if x < 4 {x + 4} else { x - 4} )},
        _ => {(1, |x| x, |x| x )}
    }
}

pub fn get_map_row(i: usize, t: usize, a: fn(usize) -> usize, b: fn(usize) -> usize) -> Vec<usize> {
    let mut map_row = vec![0; 8];
    map_row[0] = i;
    map_row[1] = a(i);
    map_row[2] = a(a(i));
    map_row[3] = a(a(a(i)));
    map_row[4] = b(i);
    map_row[5] = b(a(i));
    map_row[6] = b(a(a(i)));
    map_row[7] = b(a(a(a(i))));

    for s in 0..8 {
        map_row[s] += t;
    }

    map_row
}

pub fn process_tiles(doc: &Document, domain_name: &String, mut weights: Vec<f32>, mut tiles: Vec<Vec<u32>>, mut tilenames: Vec<String>, mut action: Vec<Vec<usize>>, mut first_occurrence: HashMap<String, usize>) -> Result<(usize, u32, Vec<f32>, Vec<Vec<u32>>, Vec<String>, Vec<Vec<usize>>, HashMap<String, usize>), Box<dyn std::error::Error>>{
    let tiles_tag = doc.descendants()
        .find(|n| n.has_tag_name("tiles"))
        .ok_or_else(|| String::from("Tag <tiles> not found in the document!"))?;

    let mut t: usize;
    let mut tilesize: u32 = 14;

    for node in tiles_tag.children().filter(|n| n.has_tag_name("tile")){
        let tile_name: String = node.attribute("name").unwrap_or("").to_string();
        let weight = node.attribute("weight")
            .and_then(|w| w.parse::<f32>().ok())
            .unwrap_or(1.0);
        let (variants, a, b) = get_cardinality_a_b_on_symmetry(node.attribute("symmetry").unwrap_or(""));

        t = action.iter().count();
        first_occurrence.insert(tile_name.clone(), t);

        for i in 0..variants {
            action.push(get_map_row(i, t,  a, b));
            weights.push(weight);
            tilenames.push(format!("{}{}", tile_name, i));
        }

        (tiles, tilesize) = load_tiles_bitmap(domain_name, &tile_name, tiles, variants, t)?;
    }

    t = action.iter().count();

    Ok((t, tilesize, weights, tiles, tilenames, action, first_occurrence))
}

pub fn load_tiles_bitmap(domain_name: &String, tile_name: &String, mut tiles: Vec<Vec<u32>>, variants: usize, t: usize) -> Result<(Vec<Vec<u32>>, u32), Box<dyn std::error::Error>>{
    let (bitmap, tilesize, _) = bitmap_utils::load_bitmap(format!("tilesets/{}/{}.png", domain_name, tile_name));
    tiles.push(bitmap);
    for i in 1..variants {
        if i <= 3 {tiles.push(rotate(&tiles[t + i - 1]));}
        if i >= 4 {tiles.push(reflect(&tiles[t + i - 4]))}
    }
    Ok((tiles, tilesize))
}

pub fn initialize_simple_tiled_model(xml_path: &str) -> Result<(usize, u32, Vec<f32>, Vec<Vec<u32>>, Vec<String>, Vec<Vec<usize>>, HashMap<String, usize>), Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(xml_path)?;
    let doc = Document::parse(&xml_content)?;
    let domain_name = match xml_path.strip_suffix(".xml"){
        Some(name) => name.to_string(),
        None => xml_path.to_string(),
    };

    let weights: Vec<f32> = Vec::new();
    let tiles: Vec<Vec<u32>> = Vec::new();
    let tilenames: Vec<String> = Vec::new();
    let action: Vec<Vec<usize>> = Vec::new();
    let first_occurrence: HashMap<String, usize> = HashMap::new();

    let (t, tilesize, weights, tiles, tilenames, action, first_occurrence) = process_tiles(&doc, &domain_name, weights, tiles, tilenames, action, first_occurrence)?;

    Ok((t, tilesize, weights, tiles, tilenames, action, first_occurrence))

}

pub fn get_propagator(xml_path: &str, t: usize, first_occurrence: HashMap<String, usize>, action: Vec<Vec<usize>>) -> Result<Vec<Vec<Vec<usize>>>, Box<dyn std::error::Error>> {
    let mut propagator: Vec<Vec<Vec<usize>>> = Vec::new();
    let mut dense_propagator: Vec<Vec<Vec<bool>>> = Vec::new();

    let xml_content = fs::read_to_string(xml_path)?;
    let doc = Document::parse(&xml_content)?;

    let neighbor_tag = doc.descendants()
        .find(|n| n.has_tag_name("neighbors"))
        .ok_or_else(|| String::from("Tag <neighbors> not found in the document!"))?;

    for neighbor in neighbor_tag.children().filter(|n| n.has_tag_name("neighbor")){
       let left: Vec<&str> = neighbor.attribute("left").unwrap().split_whitespace().collect();
       let right: Vec<&str> = neighbor.attribute("right").unwrap().split_whitespace().collect();

       let l: usize = action[first_occurrence[left[0]]][if left.len() == 1 {0} else {left[1].parse()?}];
       let d: usize = action[l][1];
       let r: usize = action[first_occurrence[right[0]]][if right.len() == 1 {0} else {right[1].parse()?}];
       let u: usize = action[r][1];

       dense_propagator[0][r][l] = true;
       dense_propagator[0][action[r][6]][action[l][6]] = true;
       dense_propagator[0][action[l][4]][action[r][4]] = true;
       dense_propagator[0][action[l][2]][action[r][2]] = true;
       
       dense_propagator[1][u][d] = true;
       dense_propagator[1][action[d][6]][action[u][6]] = true;
       dense_propagator[1][action[u][4]][action[d][4]] = true;
       dense_propagator[1][action[d][2]][action[u][2]] = true;

       for i in 0..t {
            for j in 0..t{
                dense_propagator[2][i][j] = dense_propagator[0][j][i];
                dense_propagator[3][i][j] = dense_propagator[1][j][i];
            }
       }

       let sparse_propagator: Vec<Vec<Vec<usize>>> = Vec::new();

       for d in 0..4{
        for t1 in 0..t{
            let mut sp: Vec<usize> = sparse_propagator[d][t1].clone();
            let tp: Vec<bool> = dense_propagator[d][t1].clone();

            for t2 in 0..t{
                if tp[t2]{
                    sp.push(t2);
                }
            }

            let st = sp.iter().count();
            for sti in 0..st {
                propagator[d][t1][sti] = sp[sti]
            }
        }
       }
    }

    Ok(propagator)
}