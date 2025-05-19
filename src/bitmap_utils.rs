use image::{GenericImageView, ImageBuffer, Rgba};
use std::path::Path;

pub fn load_bitmap<P: AsRef<Path>>(filename: P) -> (Vec<u32>, u32, u32) {
    // Loading the image using the image crate
    let img = image::open(filename).expect("Failed to open image");
    // Inferring dimensions
    let (width, height) = img.dimensions();
    // Converts the image to the RGBA format
    let rgba_img = img.to_rgba8();
    // Creates a mutable vector with capacity corresponding to the image size
    let mut pixels = Vec::with_capacity((width * height) as usize);

    // Converts each pixel to a u32 value
    for pixel in rgba_img.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        let a = pixel[3] as u32;

        // Packing RGBA values into a single u32
        let packed = (a << 24) | (b << 16) | (g << 8) | r;
        pixels.push(packed)
    }

    (pixels, width, height)
}

pub fn save_bitmap<P: AsRef<Path>>(filename: P, pixels: &[u32], width: u32, height: u32){
    let mut img_buffer = ImageBuffer::new(width, height);

    for y in 0..height{
        for x in 0..width{
            let index = (y * width + x) as usize;
            let pixel = pixels[index];

            let r = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = ((pixel >> 16) & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;

            img_buffer.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    img_buffer.save(filename).expect("Failed to save image");
}