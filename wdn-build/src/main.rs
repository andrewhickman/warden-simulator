use image::{ImageBuffer, RgbaImage};
use std::path::Path;

const TILE_SIZE: u32 = 64;

fn main() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();

    let input_images = vec!["assets/image/dirt.png", "assets/image/walls.png"];

    // Load all images and convert to RGBA
    let mut images = Vec::new();
    let mut total_tiles = 0u32;

    for path in &input_images {
        let full_path = workspace_root.join(path);
        let img = image::open(&full_path).unwrap().to_rgba8();

        total_tiles += (img.width() / TILE_SIZE) * (img.height() / TILE_SIZE);
        images.push(img);
    }

    let mut output: RgbaImage = ImageBuffer::new(TILE_SIZE, total_tiles * TILE_SIZE);
    let mut current_tile = 0u32;

    for img in &images {
        copy_tiles_to_output(img, &mut output, &mut current_tile);
    }

    let output_path = workspace_root.join("assets/image/tileset.png");
    output.save(&output_path).unwrap();
}

fn copy_tiles_to_output(source: &RgbaImage, output: &mut RgbaImage, current_tile: &mut u32) {
    let tiles_x = source.width() / TILE_SIZE;
    let tiles_y = source.height() / TILE_SIZE;

    for tile_y in 0..tiles_y {
        for tile_x in 0..tiles_x {
            let dest_y = *current_tile * TILE_SIZE;

            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let source_x = tile_x * TILE_SIZE + x;
                    let source_y = tile_y * TILE_SIZE + y;
                    let pixel = source.get_pixel(source_x, source_y);
                    output.put_pixel(x, dest_y + y, *pixel);
                }
            }

            *current_tile += 1;
        }
    }
}
