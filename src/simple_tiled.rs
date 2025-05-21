use roxmltree::{Document};
use core::f32;
use std::{collections::{HashMap}, fs};
use rand::Rng;
use rand::{SeedableRng, rngs::StdRng};

use crate::{array_utils::{self, reflect, rotate}, bitmap_utils};

pub struct SimpleTiledModel{
    wave: Vec<Vec<bool>>,

    propagator: Vec<Vec<Vec<usize>>>,
    compatible: Vec<Vec<Vec<usize>>>,

    stack: Vec<(isize, usize)>,

    m_x: usize,
    m_y: usize,
    t: usize,
    n: usize,

    weights: Vec<f32>,
    weight_log_weights: Vec<f32>,
    distribution: Vec<f32>,

    sums_of_ones: Vec<usize>,
    sum_of_weights: f32,
    sum_of_weight_log_weights: f32,
    starting_entropy: f32,

    sums_of_weights: Vec<f32>,
    sums_of_weight_log_weights: Vec<f32>,
    entropies: Vec<f32>,

    tiles: Vec<Vec<u32>>,
    tilenames: Vec<String>,
    tilesize: u32,

}

impl SimpleTiledModel {

    pub fn new(xml_path: &str, size: usize) -> Result<Self, Box<dyn std::error::Error>> {
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

        let (t, tilesize, weights, tiles, tilenames, action, first_occurrence) = Self::process_tiles(&doc, &domain_name, weights, tiles, tilenames, action, first_occurrence)?;

        let propagator = Self::get_propagator(&doc, t, first_occurrence, action)?;

        let weight_log_weights: Vec<f32> = weights.iter().map(|&w| w * w.ln()).collect();
        let sum_of_weights: f32 = weights.iter().sum();
        let sum_of_weight_log_weights: f32 = weight_log_weights.iter().sum();

        Ok(SimpleTiledModel { 
            wave: vec![vec![true; t]; size * size], 
            propagator: propagator, 
            compatible: vec![vec![vec![0; 4]; t]; size * size], 
            stack: Vec::new(), 
            m_x: size, 
            m_y: size, 
            t: t, 
            n: 1, 
            weights: weights, 
            weight_log_weights: weight_log_weights, 
            distribution: vec![0f32;t], 
            sums_of_ones: vec![0; size * size], 
            sum_of_weights: sum_of_weights, 
            sum_of_weight_log_weights: sum_of_weight_log_weights, 
            starting_entropy: sum_of_weights.ln() - (sum_of_weight_log_weights / sum_of_weights), 
            sums_of_weights: vec![0f32; size * size], 
            sums_of_weight_log_weights: vec![0f32; size * size], 
            entropies: vec![0f32; size * size], 
            tiles: tiles, 
            tilenames: tilenames, 
            tilesize: tilesize
        })

    }

    pub fn run(&mut self, limit: isize, seed:[u8; 32]) -> bool {
        self.clear();

        let rng = StdRng::from_seed(seed);
        let mut l = 0;
        loop {
            if limit < 0 || l >= limit {
                break;
            }

            let node = Self::next_unobserved_node(self, rng.clone());
            if node >= 0 {
                Self::observe(self, node as usize, rng.clone());
                let success = self.propagate();
                if !success {
                    return false;
                }
            } else {
                return true;
            }
            l += 1;
        }

        true
    }

    fn next_unobserved_node(&mut self, mut rng: StdRng) -> isize{
        let mut min = f32::MAX;
        let mut argmin: isize = -1;

        for i in 0..self.wave.len() {
            if i % self.m_x + self.n > self.m_x || i / self.m_x + self.n > self.m_y {
                continue;
            }

            let remaining_values = self.sums_of_ones[i];

            let entropy = self.entropies[i];

            if remaining_values > 1 && entropy <= min {
                let noise: f32 = 1E-6 * rng.random::<f32>();
                if entropy + noise < min {
                    min = entropy + noise;
                    argmin = i.cast_signed();
                }
            }
        }

        argmin
    }

    fn observe(&mut self, node: usize, mut rng: StdRng) -> bool {
       for t in 0..self.t {
            self.distribution[t] = match self.wave[node][t] {
                true => self.weights[t],
                false => 0f32,
            }
       }

       let r = array_utils::weighted_random(&self.distribution, rng.random::<f32>());
       for t in 0..self.t {
            if self.wave[node][t] != (t == r) {
                println!("{}", t);
                Self::ban(self, node, t);
            }
       }
       true
    }

    fn propagate(&mut self) -> bool {
        while let Some((position, tile)) = self.stack.pop() {
            let position_x = position % self.m_x as isize;
            let position_y = position / self.m_y as isize;

            for d in 0..4 {
                let position_x_move = position_x + Self::DX[d];
                let position_y_move = position_y + Self::DY[d];

                if position_x_move < 0 || position_y_move < 0 || position_x_move + self.n as isize > self.m_x as isize || position_y_move + self.n as isize > self.m_y as isize {
                    continue
                }

                let position_move = position_x_move + position_y_move * self.m_x as isize;

                for l in 0..self.propagator[d][tile].len() {
                    //println!("{}", self.compatible[position_move as usize][self.propagator[d][tile][l]][d]);
                    self.compatible[position_move as usize][self.propagator[d][tile][l]][d] -= 1;
                    if self.compatible[position_move as usize][self.propagator[d][tile][l]][d] == 0{
                        self.ban(position_move as usize, self.propagator[d][tile][l]);
                    }
                }

            }
        }
        return self.sums_of_ones[0] > 0
    }

    fn ban(&mut self, i: usize, t: usize){
        self.wave[i][t] = false;

        for d in 0..4 {
            self.compatible[i][t][d] = 0;
        }

        self.stack.push((i as isize, t));

        self.sums_of_ones[i] -= 1;
        self.sums_of_weights[i] -= self.weights[t];
        self.sums_of_weight_log_weights[i] -= self.weight_log_weights[t];
        self.entropies[i] = self.sums_of_weights[i].ln() - self.sums_of_weight_log_weights[i] / self.sums_of_weights[i];

    }

    fn clear(&mut self){
        for i in 0..self.wave.len() {
            for t in 0..self.t {
                self.wave[i][t] = true;

                for d in 0..4 {
                    self.compatible[i][t][d] = self.propagator[Self::OPPOSITE[d]][t].len();
                }
            }
            self.sums_of_ones[i] = self.weights.len();
            self.sums_of_weights[i] = self.sum_of_weights;
            self.sums_of_weight_log_weights[i] = self.sum_of_weight_log_weights;
            self.entropies[i] = self.starting_entropy;
        }
        
    }

    /* Helper Functions */
    fn get_cardinality_a_b_on_symmetry(symmetry: &str) -> (usize, fn(usize) -> usize, fn(usize) -> usize){
        match symmetry {
            "L" => {(4, |x| (x+1)%4, |x| if x % 2 == 0 {x + 1} else { x - 1} )},
            "T" => {(4, |x| (x+1)%4, |x| if x % 2 == 0 {x} else { 4 - x} )},
            "I" => {(2, |x| 1 - x, |x| x)},
            "\\" => {(2, |x| 1 - x, |x| 1 - x )},
            "F" => {(8, |x| if x < 4 {(x + 1) % 4} else { 4 + (x - 1) % 4}, |x| if x < 4 {x + 4} else { x - 4} )},
            _ => {(1, |x| x, |x| x )}
        }
    }

    fn get_map_row(i: usize, t: usize, a: fn(usize) -> usize, b: fn(usize) -> usize) -> Vec<usize> {
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

    fn load_tiles_bitmap(domain_name: &String, tile_name: &String, mut tiles: Vec<Vec<u32>>, variants: usize, t: usize) -> Result<(Vec<Vec<u32>>, u32), Box<dyn std::error::Error>>{
        let (bitmap, tilesize, _) = bitmap_utils::load_bitmap(format!("tilesets/{}/{}.png", domain_name, tile_name));
        tiles.push(bitmap);
        for i in 1..variants {
            if i <= 3 {tiles.push(rotate(&tiles[t + i - 1]));}
            if i >= 4 {tiles.push(reflect(&tiles[t + i - 4]))}
        }
        Ok((tiles, tilesize))
    }

    fn process_tiles(doc: &Document, domain_name: &String, mut weights: Vec<f32>, mut tiles: Vec<Vec<u32>>, mut tilenames: Vec<String>, mut action: Vec<Vec<usize>>, mut first_occurrence: HashMap<String, usize>) -> Result<(usize, u32, Vec<f32>, Vec<Vec<u32>>, Vec<String>, Vec<Vec<usize>>, HashMap<String, usize>), Box<dyn std::error::Error>>{
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
            let (variants, a, b) = Self::get_cardinality_a_b_on_symmetry(node.attribute("symmetry").unwrap_or(""));

            t = action.iter().count();
            first_occurrence.insert(tile_name.clone(), t);

            for i in 0..variants {
                action.push(Self::get_map_row(i, t,  a, b));
                weights.push(weight);
                tilenames.push(format!("{}{}", tile_name, i));
            }

            (tiles, tilesize) = Self::load_tiles_bitmap(domain_name, &tile_name, tiles, variants, t)?;
        }

        t = action.iter().count();

        Ok((t, tilesize, weights, tiles, tilenames, action, first_occurrence))
    }

    fn get_propagator(doc: &Document, t: usize, first_occurrence: HashMap<String, usize>, action: Vec<Vec<usize>>) -> Result<Vec<Vec<Vec<usize>>>, Box<dyn std::error::Error>> {

        let mut dense_propagator: Vec<Vec<Vec<bool>>> = vec![vec![vec![false; t]; t]; 4];


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

        }

        for i in 0..t {
            for j in 0..t{
                dense_propagator[2][i][j] = dense_propagator[0][j][i];
                dense_propagator[3][i][j] = dense_propagator[1][j][i];
            }
        }
        
        let mut sparse_propagator: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); t]; 4];

        for d in 0..4{
            for t1 in 0..t{
                for t2 in 0..t{
                    if dense_propagator[d][t1][t2]{
                        sparse_propagator[d][t1].push(t2);
                    }
                }
            }
        }

        Ok(sparse_propagator)
    }

    const DX: [isize; 4] = [-1, 0, 1, 0];
    const DY: [isize; 4] = [0, 1, 0, -1];
    const OPPOSITE: [usize; 4] = [2, 3, 0, 1];
}
