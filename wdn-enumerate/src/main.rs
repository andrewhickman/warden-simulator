#![warn(clippy::match_same_arms)]

use std::fs::File;
use std::io::Write;
use std::{collections::HashSet, io::BufWriter};

use wdn_enumerate::{BaseSprite, TileVariant, TopSprite};

fn main() {
    let mut unique_base_sprites = HashSet::new();
    let mut ordered_base_sprites = Vec::new();
    let mut unique_top_sprites = HashSet::new();
    let mut ordered_top_sprites = Vec::new();

    let mut base_csv = BufWriter::new(File::create("base.csv").expect("failed to create base.csv"));
    let mut top_csv = BufWriter::new(File::create("top.csv").expect("failed to create top.csv"));

    let mut consts =
        BufWriter::new(File::create("consts.rs").expect("failed to create base_consts.rs"));

    writeln!(base_csv, "result,center,east,south_east,south").unwrap();
    writeln!(top_csv, "result,center,east,south_east,south").unwrap();

    for center in TileVariant::values() {
        for east in TileVariant::values() {
            for south_east in TileVariant::values() {
                for south in TileVariant::values() {
                    let base = BaseSprite::resolve(center, east, south, south_east);
                    let top = TopSprite::resolve(center, east, south, south_east);

                    let basestr = base.to_string();
                    let topstr = top.to_string();
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

                    if unique_base_sprites.insert(base.clone()) {
                        ordered_base_sprites.push(base);
                    }
                    if unique_top_sprites.insert(top.clone()) {
                        ordered_top_sprites.push(top);
                    }
                }
            }
        }
    }

    writeln!(consts, "const {}: u16 = {};", "EMPTY", 0).unwrap();
    for (i, base) in ordered_base_sprites.iter().enumerate() {
        if base == &BaseSprite::Empty {
            continue;
        }
        if i % 16 == 0 {
            writeln!(consts).unwrap();
        }
        writeln!(consts, "const {}: u16 = {};", base, i).unwrap();
    }
    for (i, top) in ordered_top_sprites.iter().enumerate() {
        if top == &TopSprite::EMPTY {
            continue;
        }
        if (i + ordered_base_sprites.len() - 1) % 16 == 0 {
            writeln!(consts).unwrap();
        }
        writeln!(
            consts,
            "const {}: u16 = {};",
            top,
            i + ordered_base_sprites.len() - 1
        )
        .unwrap();
    }

    println!("{:?} unique base sprites", unique_base_sprites.len());
    println!("{:?} unique top sprites", unique_top_sprites.len());
}
