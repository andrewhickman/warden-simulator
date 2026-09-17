use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use wdn_enumerate::{
    BaseSprite, BaseSpriteCenter, BaseSpriteEast, BaseSpriteSouth, TopSprite, TopSpriteCenter,
    TopSpriteEast, TopSpriteSouth, TopSpriteSouthEast,
};

pub(crate) const TILE_WIDTH: u32 = 200;
pub(crate) const TILE_HEIGHT: u32 = 400;
const SPRITES_PER_ROW: u32 = 16;
const INKSCAPE_NS: &str = "http://www.inkscape.org/namespaces/inkscape";

/// Normalizes and deduplicates `base_sprites` and `top_sprites`, then renders them into a
/// single spritesheet SVG (base tiles first, followed by top tiles) by layering the parts
/// copied from `base_parts_path`/`top_parts_path`, writing the result to `output_path`.
pub fn generate_spritesheet(
    base_sprites: &[BaseSprite],
    base_parts_path: &Path,
    top_sprites: &[TopSprite],
    top_parts_path: &Path,
    output_path: &Path,
) -> std::io::Result<()> {
    let base_parts = extract_labeled_parts(&fs::read_to_string(base_parts_path)?);
    let top_parts = extract_labeled_parts(&fs::read_to_string(top_parts_path)?);

    let base_tiles = normalized_unique(base_sprites, BaseSprite::normalize);
    let mut top_tiles = normalized_unique(top_sprites, TopSprite::normalize);
    // The empty top sprite duplicates the empty base sprite already first in the sheet.
    if top_tiles.first() == Some(&TopSprite::EMPTY) {
        top_tiles.remove(0);
    }

    let mut tiles: Vec<(String, Vec<String>)> = base_tiles
        .iter()
        .map(|&sprite| {
            let labels = [
                center_label(sprite.center),
                south_label(sprite.south),
                east_label(sprite.east),
            ];
            (
                format!("{sprite:?}"),
                resolve_layers(&labels, &base_parts, base_parts_path),
            )
        })
        .collect();
    tiles.extend(top_tiles.iter().map(|&sprite| {
        let labels = [
            center_top_label(sprite.center),
            south_top_label(sprite.south),
            south_east_top_label(sprite.south_east),
            east_top_label(sprite.east),
        ];
        (
            format!("{sprite:?}"),
            resolve_layers(&labels, &top_parts, top_parts_path),
        )
    }));

    let svg = build_svg(&tiles);
    fs::write(output_path, &svg)?;

    let pixmap = render_to_pixmap(&svg);
    pixmap
        .save_png(output_path.with_extension("png"))
        .expect("failed to save spritesheet png");
    // Base and top tiles are unrelated sprite kinds, so check each for duplicates separately;
    // a coincidental pixel match between the two isn't a missing `normalize()` rule.
    report_visual_duplicates(&pixmap, &tiles[..base_tiles.len()], 0);
    report_visual_duplicates(&pixmap, &tiles[base_tiles.len()..], base_tiles.len());

    Ok(())
}

/// Normalizes `sprites`, keeping only the first occurrence of each distinct value.
fn normalized_unique<T: Copy + Eq + std::hash::Hash>(
    sprites: &[T],
    normalize: impl Fn(T) -> T,
) -> Vec<T> {
    let mut seen = HashSet::new();
    sprites
        .iter()
        .copied()
        .map(normalize)
        .filter(|sprite| seen.insert(*sprite))
        .collect()
}

/// Looks up each of `labels` in `parts`, cloning the matching SVG fragments.
pub(crate) fn resolve_layers(
    labels: &[&str],
    parts: &HashMap<String, String>,
    parts_path: &Path,
) -> Vec<String> {
    labels
        .iter()
        .map(|label| {
            parts
                .get(*label)
                .unwrap_or_else(|| panic!("missing part `{label}` in {}", parts_path.display()))
                .clone()
        })
        .collect()
}

/// Lays `tiles` out in a grid, returning the resulting spritesheet SVG source.
fn build_svg(tiles: &[(String, Vec<String>)]) -> String {
    let columns = SPRITES_PER_ROW;
    let rows = (tiles.len() as u32).div_ceil(columns);
    let sheet_width = columns * TILE_WIDTH;
    let sheet_height = rows * TILE_HEIGHT;

    let mut groups = Vec::with_capacity(tiles.len());
    for (index, (name, layers)) in tiles.iter().enumerate() {
        let x = (index as u32 % columns) * TILE_WIDTH;
        let y = (index as u32 / columns) * TILE_HEIGHT;
        let name = escape_xml_attribute(name);

        let mut group =
            format!("  <g inkscape:label=\"{name}\" transform=\"translate({x},{y})\">\n");
        for layer in layers {
            group.push_str(layer);
            group.push('\n');
        }
        group.push_str("  </g>\n");
        groups.push(group);
    }
    // Reverse the tiles' document order; each keeps its original grid position.
    let body: String = groups.into_iter().rev().collect();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n\
         <svg width=\"{sheet_width}\" height=\"{sheet_height}\" viewBox=\"0 0 {sheet_width} {sheet_height}\" \
         xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"{INKSCAPE_NS}\" \
         xmlns:sodipodi=\"http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd\">\n{body}</svg>\n"
    )
}

pub(crate) fn render_to_pixmap(svg: &str) -> Pixmap {
    let tree = Tree::from_str(svg, &Options::default()).expect("failed to parse generated svg");
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("invalid pixmap size");
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    pixmap
}

/// Groups `tiles` by their rendered pixels (as laid out by [`build_svg`], starting at grid
/// index `first_index`) and prints any group sharing an identical image, which would
/// indicate a missing `normalize()` rule.
fn report_visual_duplicates(pixmap: &Pixmap, tiles: &[(String, Vec<String>)], first_index: usize) {
    let columns = SPRITES_PER_ROW;
    let stride = pixmap.width() as usize * 4;
    let mut tiles_by_pixels: HashMap<Vec<u8>, Vec<&str>> = HashMap::new();

    for (offset, (name, _)) in tiles.iter().enumerate() {
        let index = (first_index + offset) as u32;
        let tile_x = (index % columns) as usize * TILE_WIDTH as usize * 4;
        let tile_y = (index / columns) as usize * TILE_HEIGHT as usize;

        let mut pixels = Vec::with_capacity(TILE_WIDTH as usize * TILE_HEIGHT as usize * 4);
        for row in 0..TILE_HEIGHT as usize {
            let start = (tile_y + row) * stride + tile_x;
            pixels.extend_from_slice(&pixmap.data()[start..start + TILE_WIDTH as usize * 4]);
        }

        tiles_by_pixels.entry(pixels).or_default().push(name);
    }

    let mut duplicate_count = 0;
    for group in tiles_by_pixels.values() {
        if group.len() > 1 {
            println!("visually identical sprites: {}", group.join(", "));
            duplicate_count += group.len();
        }
    }

    println!("{duplicate_count} visually identical sprites");
}

pub(crate) fn center_label(center: BaseSpriteCenter) -> &'static str {
    match center {
        BaseSpriteCenter::None => "CenterNone",
        BaseSpriteCenter::WallCorner => "CenterWallCorner",
        BaseSpriteCenter::WallVertical => "CenterWallVertical",
        BaseSpriteCenter::WallHorizontal => "CenterWallHorizontal",
        BaseSpriteCenter::WallInverseCorner => "CenterWallInverseCorner",
        BaseSpriteCenter::WallFull => "CenterWallFull",
        BaseSpriteCenter::StairN => "CenterStairN",
        BaseSpriteCenter::StairNFull => "CenterStairNFull",
        BaseSpriteCenter::StairS => "CenterStairS",
        BaseSpriteCenter::StairSFull => "CenterStairSFull",
        BaseSpriteCenter::StairE => "CenterStairE",
        BaseSpriteCenter::StairEFull => "CenterStairEFull",
        BaseSpriteCenter::StairW => "CenterStairW",
        BaseSpriteCenter::StairWFull => "CenterStairWFull",
    }
}

pub(crate) fn south_label(south: BaseSpriteSouth) -> &'static str {
    match south {
        BaseSpriteSouth::None => "SouthNone",
        BaseSpriteSouth::Door => "SouthDoor",
    }
}

pub(crate) fn east_label(east: BaseSpriteEast) -> &'static str {
    match east {
        BaseSpriteEast::None => "EastNone",
        BaseSpriteEast::Door => "EastDoor",
        BaseSpriteEast::StairN => "EastStairN",
        BaseSpriteEast::StairS => "EastStairS",
    }
}

pub(crate) fn center_top_label(center: TopSpriteCenter) -> &'static str {
    match center {
        TopSpriteCenter::None => "CenterNone",
        TopSpriteCenter::WallCorner => "CenterWallCorner",
        TopSpriteCenter::WallVertical => "CenterWallVertical",
        TopSpriteCenter::WallHorizontal => "CenterWallHorizontal",
        TopSpriteCenter::WallInverseCorner => "CenterWallInverseCorner",
        TopSpriteCenter::WallFull => "CenterWallFull",
        TopSpriteCenter::Door => "CenterDoor",
        TopSpriteCenter::StairS => "CenterStairS",
        TopSpriteCenter::StairSFull => "CenterStairSFull",
    }
}

pub(crate) fn south_top_label(south: TopSpriteSouth) -> &'static str {
    match south {
        TopSpriteSouth::None => "SouthNone",
        TopSpriteSouth::Door => "SouthDoor",
        TopSpriteSouth::WallCorner => "SouthCorner",
        TopSpriteSouth::WallHorizontal => "SouthHorizontal",
        TopSpriteSouth::StairN => "SouthStairN",
        TopSpriteSouth::StairNFull => "SouthStairNFull",
        TopSpriteSouth::StairE => "SouthStairE",
        TopSpriteSouth::StairEFull => "SouthStairEFull",
        TopSpriteSouth::StairW => "SouthStairW",
        TopSpriteSouth::StairWFull => "SouthStairWFull",
    }
}

pub(crate) fn south_east_top_label(south_east: TopSpriteSouthEast) -> &'static str {
    match south_east {
        TopSpriteSouthEast::None => "SouthEastNone",
        TopSpriteSouthEast::StairN => "SouthEastStairN",
    }
}

pub(crate) fn east_top_label(east: TopSpriteEast) -> &'static str {
    match east {
        TopSpriteEast::None => "EastNone",
        TopSpriteEast::Door => "EastDoor",
        TopSpriteEast::StairS => "EastStairS",
    }
}

fn escape_xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Collects every named part group (`inkscape:label`) keyed by that label, forcing each
/// to render since the source file hides all but one part per layer with `display:none`.
pub(crate) fn extract_labeled_parts(svg: &str) -> HashMap<String, String> {
    let doc = roxmltree::Document::parse(svg).expect("failed to parse parts svg");
    let mut parts = HashMap::new();

    for node in doc.descendants() {
        if !node.has_tag_name("g") {
            continue;
        }
        // Skip the top-level Center/South/East layer groups; only keep the named parts.
        if node.attribute((INKSCAPE_NS, "groupmode")) == Some("layer") {
            continue;
        }
        let Some(label) = node.attribute((INKSCAPE_NS, "label")) else {
            continue;
        };

        let raw = &svg[node.range()];
        let visible = raw.replacen("display:none", "display:inline", 1);
        parts.insert(label.to_string(), visible);
    }

    parts
}
