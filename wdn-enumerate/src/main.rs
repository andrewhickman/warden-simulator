#![warn(clippy::match_same_arms)]

use std::fs::File;
use std::io::Write;
use std::{collections::HashSet, io::BufWriter};

use wdn_enumerate::{BaseSprite, TileVariant, TopSprite};

mod spritesheet;

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
                    let top = TopSprite::resolve(center, east, south, south_east, base);

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

    generate_ids(&ordered_base_sprites, &ordered_top_sprites);
    // spritesheet::generate(&ordered_base_sprites, &ordered_top_sprites);
}

/// Writes `ids.rs` with `id()` match arms using the same numbering as `consts.rs`.
fn generate_ids(ordered_base_sprites: &[BaseSprite], ordered_top_sprites: &[TopSprite]) {
    let mut ids = BufWriter::new(
        File::create("wdn-enumerate/src/ids.rs")
            .expect("failed to create wdn-enumerate/src/ids.rs"),
    );

    writeln!(ids, "use super::*;").unwrap();
    writeln!(ids).unwrap();

    writeln!(ids, "impl BaseSprite {{").unwrap();
    writeln!(ids, "    pub fn id(&self) -> u16 {{").unwrap();
    writeln!(ids, "        match self {{").unwrap();
    writeln!(ids, "            BaseSprite::Empty => 0,").unwrap();
    for (i, base) in ordered_base_sprites.iter().enumerate() {
        if base == &BaseSprite::Empty {
            continue;
        }
        writeln!(ids, "            {base:?} => {i},").unwrap();
    }
    writeln!(
        ids,
        "            _ => unreachable!(\"id() called on a sprite not produced by the enumerator: {{self:?}}\"),"
    )
    .unwrap();
    writeln!(ids, "        }}").unwrap();
    writeln!(ids, "    }}").unwrap();
    writeln!(ids, "}}").unwrap();
    writeln!(ids).unwrap();

    writeln!(ids, "impl TopSprite {{").unwrap();
    writeln!(ids, "    pub fn id(&self) -> usize {{").unwrap();
    writeln!(ids, "        match self {{").unwrap();
    for (i, top) in ordered_top_sprites.iter().enumerate() {
        let id = if top == &TopSprite::EMPTY {
            0
        } else {
            i + ordered_base_sprites.len() - 1
        };
        writeln!(ids, "            {top:?} => {id},").unwrap();
    }
    writeln!(
        ids,
        "            _ => unreachable!(\"id() called on a sprite not produced by the enumerator: {{self:?}}\"),"
    )
    .unwrap();
    writeln!(ids, "        }}").unwrap();
    writeln!(ids, "    }}").unwrap();
    writeln!(ids, "}}").unwrap();
}
