#![warn(clippy::match_same_arms)]

use std::fs::File;
use std::io::Write;
use std::{
    collections::{HashMap, HashSet},
    io::BufWriter,
};

use wdn_enumerate::mask::{Classification, SpriteMask, TILE_HEIGHT, TILE_WIDTH};
use wdn_enumerate::{BaseSprite, TileVariant, TopSprite};

fn main() {
    let mut unique_base_sprites = HashSet::new();
    let mut ordered_base_sprites = Vec::new();
    let mut unique_top_sprites = HashSet::new();
    let mut ordered_top_sprites = Vec::new();
    let mut unique_pairs = HashSet::new();

    let sprite_mask = SpriteMask::load("assets/image/walls.png", "masked_walls.png");

    let mut base_csv = BufWriter::new(File::create("base.csv").expect("failed to create base.csv"));
    let mut top_csv = BufWriter::new(File::create("top.csv").expect("failed to create top.csv"));
    let mut overlap_csv =
        BufWriter::new(File::create("overlap.csv").expect("failed to create overlap.csv"));

    let mut consts =
        BufWriter::new(File::create("consts.rs").expect("failed to create base_consts.rs"));

    writeln!(base_csv, "result,center,east,south_east,south").unwrap();
    writeln!(top_csv, "result,center,east,south_east,south").unwrap();
    writeln!(overlap_csv, "base,top,classification").unwrap();

    let mut empty_count = 0usize;
    let mut disjoint_count = 0usize;
    let mut overlap_count = 0usize;
    let mut top_overlapping_bases: HashMap<TopSprite, Vec<BaseSprite>> = HashMap::new();

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

                    if unique_pairs.insert((base, top)) {
                        let classification = sprite_mask.classify(base.id() as usize, top.id());
                        writeln!(overlap_csv, "{basestr},{topstr},{classification:?}").unwrap();
                        match classification {
                            Classification::Empty => empty_count += 1,
                            Classification::Disjoint => disjoint_count += 1,
                            Classification::Overlap => overlap_count += 1,
                        }

                        if classification == Classification::Overlap {
                            top_overlapping_bases.entry(top).or_default().push(base);
                        }
                    }

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
    println!("{:?} unique base/top pairs", unique_pairs.len());
    println!("{empty_count} empty, {disjoint_count} disjoint, {overlap_count} overlap");

    println!("top sprites that ever overlap with a base sprite:");
    for top in &ordered_top_sprites {
        let Some(bases) = top_overlapping_bases.get(top) else {
            continue;
        };
        println!("  {top}");
        for base in bases {
            println!("    {base}");
        }
    }

    render_overlaps(&sprite_mask, &ordered_top_sprites, &top_overlapping_bases);
}

/// Renders every overlapping (base, top) pair into a grid PNG, one row per top sprite.
fn render_overlaps(
    sprite_mask: &SpriteMask,
    ordered_top_sprites: &[TopSprite],
    top_overlapping_bases: &HashMap<TopSprite, Vec<BaseSprite>>,
) {
    let rows: Vec<(&TopSprite, &Vec<BaseSprite>)> = ordered_top_sprites
        .iter()
        .filter_map(|top| top_overlapping_bases.get(top).map(|bases| (top, bases)))
        .collect();
    let columns = rows.iter().map(|(_, bases)| bases.len()).max().unwrap_or(0) as u32;
    let row_count = rows.len() as u32;

    let mut canvas = image::RgbaImage::new(columns * TILE_WIDTH, row_count * TILE_HEIGHT);
    for (row, (top, bases)) in rows.iter().enumerate() {
        for (column, base) in bases.iter().enumerate() {
            let tile = sprite_mask.composite(base.id() as usize, top.id());
            image::imageops::replace(
                &mut canvas,
                &tile,
                column as i64 * TILE_WIDTH as i64,
                row as i64 * TILE_HEIGHT as i64,
            );
        }
    }

    canvas
        .save("overlaps.png")
        .expect("failed to write overlaps.png");
}
