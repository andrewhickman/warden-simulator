#![warn(clippy::match_same_arms)]

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{collections::HashSet, io::BufWriter};

use wdn_enumerate::{TileVariant, resolve_sprites};

mod spritesheet;

fn main() {
    let mut unique_base_sprites = HashSet::new();
    let mut ordered_base_sprites = Vec::new();
    let mut unique_top_sprites = HashSet::new();
    let mut ordered_top_sprites = Vec::new();

    let mut base_csv = BufWriter::new(File::create("base.csv").expect("failed to create base.csv"));
    let mut top_csv = BufWriter::new(File::create("top.csv").expect("failed to create top.csv"));

    let mut base_consts =
        BufWriter::new(File::create("base_consts.rs").expect("failed to create base_consts.rs"));
    let mut top_consts =
        BufWriter::new(File::create("top_consts.rs").expect("failed to create top_consts.rs"));

    writeln!(base_csv, "result,center,east,south_east,south").unwrap();
    writeln!(top_csv, "result,center,east,south_east,south").unwrap();

    for center in TileVariant::values() {
        for east in TileVariant::values() {
            for south_east in TileVariant::values() {
                for south in TileVariant::values() {
                    let (base, top) = resolve_sprites(
                        center,
                        TileVariant::Empty,
                        TileVariant::Empty,
                        east,
                        south_east,
                        south,
                    );

                    for north in TileVariant::values() {
                        for north_east in TileVariant::values() {
                            let (alt_base, alt_top) =
                                resolve_sprites(center, north, north_east, east, south_east, south);
                            assert_eq!(base, alt_base);
                            assert_eq!(top, alt_top);
                        }
                    }

                    let basestr = format!("BASE_{:?}", base).to_uppercase();
                    let topstr = format!("TOP_{:?}", top).to_uppercase();
                    writeln!(
                        top_csv,
                        "{},{:?},{:?},{:?},{:?}",
                        topstr, center, east, south_east, south
                    )
                    .unwrap();

                    writeln!(
                        base_csv,
                        "{},{:?},{:?},{:?},{:?}",
                        basestr, center, east, south_east, south
                    )
                    .unwrap();

                    if unique_base_sprites.insert(base) {
                        writeln!(base_consts, "const {}: u16 = {};", basestr, base.id()).unwrap();
                        ordered_base_sprites.push(base);
                    }
                    if unique_top_sprites.insert(top) {
                        writeln!(top_consts, "const {}: u16 = {};", topstr, top.id()).unwrap();
                        ordered_top_sprites.push(top);
                    }
                }
            }
        }
    }

    println!("{:?} unique base sprites", unique_base_sprites.len());
    println!("{:?} unique top sprites", unique_top_sprites.len());

    spritesheet::generate_spritesheet(
        &ordered_base_sprites,
        Path::new("assets/image/wall_base_parts.svg"),
        &ordered_top_sprites,
        Path::new("assets/image/wall_top_parts.svg"),
        Path::new("assets/image/wall_spritesheet.svg"),
    )
    .expect("failed to generate wall spritesheet");

    spritesheet::generate_sprite_ids(
        &ordered_base_sprites,
        &ordered_top_sprites,
        Path::new("wdn-enumerate/src/sprite_id.rs"),
    )
    .expect("failed to generate sprite ids");
}
