#![warn(clippy::match_same_arms)]

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{collections::HashSet, io::BufWriter};

use wdn_enumerate::{BaseSprite, TileVariant, TopSprite, resolve_sprites};

mod spritesheet;

fn main() {
    // let (base, top) = resolve_sprites(
    //     TileVariant::Wall,
    //     TileVariant::Empty,
    //     TileVariant::Empty,
    //     TileVariant::StairS,
    //     TileVariant::Wall,
    //     TileVariant::Wall,
    // );
    // println!("base: {:?}, top: {:?}", base, top);
    // return;

    let mut unique_base_sprites = HashSet::new();
    let mut ordered_base_sprites = Vec::new();
    let mut unique_top_sprites = HashSet::new();
    let mut ordered_top_sprites = Vec::new();
    let mut unique_pairs = HashSet::new();

    let mut base_csv = BufWriter::new(File::create("base.csv").expect("failed to create base.csv"));
    let mut top_csv = BufWriter::new(File::create("top.csv").expect("failed to create top.csv"));

    let mut base_consts =
        BufWriter::new(File::create("consts.rs").expect("failed to create base_consts.rs"));

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
                    let simplified_top = top/*.simplify()*/;

                    for north in TileVariant::values() {
                        for north_east in TileVariant::values() {
                            let (alt_base, alt_top) =
                                resolve_sprites(center, north, north_east, east, south_east, south);
                            assert_eq!(base, alt_base);
                            assert_eq!(top, alt_top);
                        }
                    }

                    let basestr = if base == BaseSprite::EMPTY {
                        "EMPTY".to_owned()
                    } else {
                        format!("BASE_{:?}", base).to_uppercase()
                    };
                    let topstr = if simplified_top == TopSprite::EMPTY {
                        "EMPTY".to_owned()
                    } else {
                        format!("TOP_{:?}", simplified_top).to_uppercase()
                    };
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

                    // Contiguity analysis needs the pre-simplify (normalize-only) top sprite
                    // paired with its base, so that regenerating `simplify()` never analyzes
                    // its own prior output (which would make each run a moving target).
                    unique_pairs.insert((base, top));

                    if unique_base_sprites.insert(base) {
                        // writeln!(base_consts, "const {}: u16 = {};", basestr, base.id()).unwrap();
                        ordered_base_sprites.push(base);
                    }
                    if unique_top_sprites.insert(simplified_top) {
                        // writeln!(
                        //     top_consts,
                        //     "const {}: u16 = {};",
                        //     topstr,
                        //     simplified_top.id()
                        // )
                        // .unwrap();
                        ordered_top_sprites.push(simplified_top);
                    }
                }
            }
        }
    }

    writeln!(base_consts, "const {}: u16 = {};", "EMPTY", 0).unwrap();
    for (i, base) in ordered_base_sprites.iter().enumerate() {
        if base == &BaseSprite::EMPTY {
            continue;
        }
        let basestr = format!("BASE_{:?}", base).to_uppercase();
        writeln!(base_consts, "const {}: u16 = {};", basestr, i).unwrap();
    }
    for (i, top) in ordered_top_sprites.iter().enumerate() {
        if top == &TopSprite::EMPTY {
            continue;
        }
        let topstr = format!("TOP_{:?}", top).to_uppercase();
        writeln!(
            base_consts,
            "const {}: u16 = {};",
            topstr,
            i + ordered_base_sprites.len() - 1
        )
        .unwrap();
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
}
