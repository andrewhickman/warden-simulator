#![warn(clippy::match_same_arms)]

use std::collections::HashSet;

use wdn_enumerate::{TileVariant, resolve_sprites};

fn main() {
    let mut unique_base_sprites = HashSet::new();
    let mut unique_top_sprites = HashSet::new();

    for center in TileVariant::values() {
        for north in TileVariant::values() {
            for north_east in TileVariant::values() {
                for east in TileVariant::values() {
                    for south_east in TileVariant::values() {
                        for south in TileVariant::values() {
                            let (base, top) =
                                resolve_sprites(center, north, north_east, east, south_east, south);
                            if unique_base_sprites.insert(base) {
                                // println!("{:?}", base);
                            }
                            if unique_top_sprites.insert(top) {
                                println!(
                                    "center: {:?}, north: {:?}, north_east: {:?}, east: {:?}, south_east: {:?}, south: {:?}",
                                    center, north, north_east, east, south_east, south
                                );
                                println!("{:?}", top);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{:?} unique base sprites", unique_base_sprites.len());
    println!("{:?} unique top sprites", unique_top_sprites.len());
}
