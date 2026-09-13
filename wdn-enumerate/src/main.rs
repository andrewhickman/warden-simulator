#![warn(clippy::match_same_arms)]

use std::collections::HashSet;
use std::path::Path;

use wdn_enumerate::{TileVariant, resolve_sprites};

mod spritesheet;

fn main() {
    let mut unique_base_sprites = HashSet::new();
    let mut ordered_base_sprites = Vec::new();
    let mut unique_top_sprites = HashSet::new();
    let mut ordered_top_sprites = Vec::new();

    for center in TileVariant::values() {
        for north in TileVariant::values() {
            for north_east in TileVariant::values() {
                for east in TileVariant::values() {
                    for south_east in TileVariant::values() {
                        for south in TileVariant::values() {
                            let (base, top) =
                                resolve_sprites(center, north, north_east, east, south_east, south);
                            if unique_base_sprites.insert(base) {
                                println!(
                                    "center: {:?}, north: {:?}, north_east: {:?}, east: {:?}, south_east: {:?}, south: {:?}",
                                    center, north, north_east, east, south_east, south
                                );
                                println!("sprite: {:?}", base);
                                ordered_base_sprites.push(base);
                            }
                            if unique_top_sprites.insert(top) {
                                ordered_top_sprites.push(top);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{:?} unique base sprites", unique_base_sprites.len());
    println!("{:?} unique top sprites", unique_top_sprites.len());

    spritesheet::generate_base_spritesheet(
        &ordered_base_sprites,
        Path::new("assets/image/wall_base_parts.svg"),
        Path::new("assets/image/wall_base_spritesheet.svg"),
    )
    .expect("failed to generate wall base spritesheet");

    spritesheet::generate_top_spritesheet(
        &ordered_top_sprites,
        Path::new("assets/image/wall_top_parts.svg"),
        Path::new("assets/image/wall_top_spritesheet.svg"),
    )
    .expect("failed to generate wall top spritesheet");
}
