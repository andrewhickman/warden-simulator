use image::{ImageBuffer, RgbaImage};
use std::path::Path;

const TILE_WIDTH: u32 = 32;
const TILE_HEIGHT: u32 = 64;

fn main() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();

    let input_images = vec!["assets/image/dirt.png", "assets/image/walls.png"];

    let mut images = Vec::new();
    let mut total_tiles = 0u32;

    for path in &input_images {
        let full_path = workspace_root.join(path);
        let img = image::open(&full_path).unwrap().to_rgba8();

        total_tiles += (img.width() / TILE_WIDTH) * (img.height() / TILE_HEIGHT);
        images.push(img);
    }

    let mut output: RgbaImage = ImageBuffer::new(TILE_WIDTH, total_tiles * TILE_HEIGHT);
    let mut current_tile = 0u32;

    for img in &images {
        copy_tiles_to_output(img, &mut output, &mut current_tile);
    }

    let output_path = workspace_root.join("assets/image/tileset.png");
    output.save(&output_path).unwrap();
}

fn copy_tiles_to_output(source: &RgbaImage, output: &mut RgbaImage, current_tile: &mut u32) {
    let tiles_x = source.width() / TILE_WIDTH;
    let tiles_y = source.height() / TILE_HEIGHT;

    for tile_y in 0..tiles_y {
        for tile_x in 0..tiles_x {
            let dest_y = *current_tile * TILE_HEIGHT;

            for y in 0..TILE_HEIGHT {
                for x in 0..TILE_WIDTH {
                    let source_x = tile_x * TILE_WIDTH + x;
                    let source_y = tile_y * TILE_HEIGHT + y;
                    let pixel = source.get_pixel(source_x, source_y);
                    output.put_pixel(x, dest_y + y, *pixel);
                }
            }

            *current_tile += 1;
        }
    }
}
