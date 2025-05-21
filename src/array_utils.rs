pub fn rotate(array: &[u32]) -> Vec<u32> {
    let total_len = array.len();
    // Per una griglia quadrata, size = √total_len
    let size = (total_len as f64).sqrt() as usize;
    
    // Assicuriamoci che sia effettivamente una griglia quadrata
    assert_eq!(size * size, total_len, "L'array non rappresenta una griglia quadrata");
    
    tile(|x, y| array[size - 1 - y + x * size], size)
}

pub fn reflect(array: &[u32]) -> Vec<u32> {
    let total_len = array.len();
    let size = (total_len as f64).sqrt() as usize;
    
    assert_eq!(size * size, total_len, "L'array non rappresenta una griglia quadrata");
    
    tile(|x, y| array[size - 1 - x + y * size], size)
}


fn tile<F>(f: F, size: usize) -> Vec<u32> 
where 
    F: Fn(usize, usize) -> u32
{
    let mut result = vec![0; size * size];
    for y in 0..size {
        for x in 0..size {
            result[x + y * size] = f(x, y);
        }
    }
    result
}

pub fn weighted_random(weights: &[f32], r: f32) -> usize {
    let total: f32 = weights.iter().sum();
    let threshold = r * total;
    
    let mut partial_sum = 0.0;
    for i in 0..weights.len() {
        partial_sum += weights[i];
        if partial_sum >= threshold {
            return i;
        }
    }
    0
}