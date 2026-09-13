use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use wdn_enumerate::{
    BaseSprite, BaseSpriteCenter, BaseSpriteEast, BaseSpriteSouth, TopSprite, TopSpriteCenter,
    TopSpriteEast, TopSpriteSouth, TopSpriteSouthEast,
};

const TILE_WIDTH: u32 = 200;
const TILE_HEIGHT: u32 = 400;
const SPRITES_PER_ROW: u32 = 16;
const INKSCAPE_NS: &str = "http://www.inkscape.org/namespaces/inkscape";

/// Renders `sprites` into a spritesheet SVG by layering the center/south/east parts
/// copied from `parts_path`, and writes the result to `output_path`.
pub fn generate_base_spritesheet(
    sprites: &[BaseSprite],
    parts_path: &Path,
    output_path: &Path,
) -> std::io::Result<()> {
    generate_spritesheet(sprites, parts_path, output_path, |sprite| {
        vec![
            center_label(sprite.center),
            south_label(sprite.south),
            east_label(sprite.east),
        ]
    })
}

/// Renders `sprites` into a spritesheet SVG by layering the center/south/south-east/east
/// parts copied from `parts_path`, and writes the result to `output_path`.
pub fn generate_top_spritesheet(
    sprites: &[TopSprite],
    parts_path: &Path,
    output_path: &Path,
) -> std::io::Result<()> {
    generate_spritesheet(sprites, parts_path, output_path, |sprite| {
        vec![
            center_top_label(sprite.center),
            south_top_label(sprite.south),
            south_east_top_label(sprite.south_east),
            east_top_label(sprite.east),
        ]
    })
}

fn generate_spritesheet<T: Debug + Copy>(
    sprites: &[T],
    parts_path: &Path,
    output_path: &Path,
    layer_labels: impl Fn(T) -> Vec<&'static str>,
) -> std::io::Result<()> {
    let parts_svg = fs::read_to_string(parts_path)?;
    let parts = extract_labeled_parts(&parts_svg);

    let columns = SPRITES_PER_ROW;
    let rows = (sprites.len() as u32).div_ceil(columns);
    let sheet_width = columns * TILE_WIDTH;
    let sheet_height = rows * TILE_HEIGHT;

    let mut body = String::new();
    for (index, sprite) in sprites.iter().enumerate() {
        let x = (index as u32 % columns) * TILE_WIDTH;
        let y = (index as u32 / columns) * TILE_HEIGHT;
        let name = escape_xml_attribute(&format!("{sprite:?}"));

        body.push_str(&format!(
            "  <g inkscape:label=\"{name}\" transform=\"translate({x},{y})\">\n"
        ));
        for label in layer_labels(*sprite) {
            let part = parts
                .get(label)
                .unwrap_or_else(|| panic!("missing part `{label}` in {}", parts_path.display()));
            body.push_str(part);
            body.push('\n');
        }
        body.push_str("  </g>\n");
    }

    let svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n\
         <svg width=\"{sheet_width}\" height=\"{sheet_height}\" viewBox=\"0 0 {sheet_width} {sheet_height}\" \
         xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"{INKSCAPE_NS}\" \
         xmlns:sodipodi=\"http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd\">\n{body}</svg>\n"
    );

    fs::write(output_path, &svg)?;

    let pixmap = render_to_pixmap(&svg);
    pixmap
        .save_png(output_path.with_extension("png"))
        .expect("failed to save spritesheet png");
    report_visual_duplicates(&pixmap, sprites, columns);

    Ok(())
}

fn render_to_pixmap(svg: &str) -> Pixmap {
    let tree = Tree::from_str(svg, &Options::default()).expect("failed to parse generated svg");
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("invalid pixmap size");
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    pixmap
}

/// Groups sprite tiles by their rendered pixels and prints any group sharing an
/// identical image, which would indicate visually redundant sprite variants.
fn report_visual_duplicates<T: Debug>(pixmap: &Pixmap, sprites: &[T], columns: u32) {
    let stride = pixmap.width() as usize * 4;
    let mut tiles_by_pixels: HashMap<Vec<u8>, Vec<String>> = HashMap::new();

    for (index, sprite) in sprites.iter().enumerate() {
        let tile_x = (index as u32 % columns) as usize * TILE_WIDTH as usize * 4;
        let tile_y = (index as u32 / columns) as usize * TILE_HEIGHT as usize;

        let mut pixels = Vec::with_capacity(TILE_WIDTH as usize * TILE_HEIGHT as usize * 4);
        for row in 0..TILE_HEIGHT as usize {
            let start = (tile_y + row) * stride + tile_x;
            pixels.extend_from_slice(&pixmap.data()[start..start + TILE_WIDTH as usize * 4]);
        }

        tiles_by_pixels
            .entry(pixels)
            .or_default()
            .push(format!("{sprite:?}"));
    }

    let mut duplicate_count = 0;
    for names in tiles_by_pixels.values() {
        if names.len() > 1 {
            println!("visually identical sprites: {}", names.join(", "));
            duplicate_count += names.len();
        }
    }

    println!("{duplicate_count} visually identical sprites");
}

fn center_label(center: BaseSpriteCenter) -> &'static str {
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

fn south_label(south: BaseSpriteSouth) -> &'static str {
    match south {
        BaseSpriteSouth::None => "SouthNone",
        BaseSpriteSouth::Door => "SouthDoor",
    }
}

fn east_label(east: BaseSpriteEast) -> &'static str {
    match east {
        BaseSpriteEast::None => "EastNone",
        BaseSpriteEast::Door => "EastDoor",
        BaseSpriteEast::StairN => "EastStairN",
        BaseSpriteEast::StairS => "EastStairS",
    }
}

fn center_top_label(center: TopSpriteCenter) -> &'static str {
    match center {
        TopSpriteCenter::None => "CenterNone",
        TopSpriteCenter::WallCorner => "CenterWallCorner",
        TopSpriteCenter::WallVertical => "CenterWallVertical",
        TopSpriteCenter::WallHorizontal => "CenterWallHorizontal",
        TopSpriteCenter::WallInverseCorner => "CenterWallInverseCorner",
        TopSpriteCenter::WallFull => "CenterWallFull",
        TopSpriteCenter::Door => "Corner_Door",
    }
}

fn south_top_label(south: TopSpriteSouth) -> &'static str {
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

fn south_east_top_label(south_east: TopSpriteSouthEast) -> &'static str {
    match south_east {
        TopSpriteSouthEast::None => "SouthEastNone",
        TopSpriteSouthEast::StairN => "SouthEastStairN",
        TopSpriteSouthEast::StairW => "SouthEastStairW",
        TopSpriteSouthEast::StairWFull => "SouthEastStairWFull",
    }
}

fn east_top_label(east: TopSpriteEast) -> &'static str {
    match east {
        TopSpriteEast::None => "EastNone",
        TopSpriteEast::Door => "EastDoor",
        TopSpriteEast::StairW => "EastStairW",
        TopSpriteEast::StairWFull => "EastStairWFull",
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
fn extract_labeled_parts(svg: &str) -> HashMap<String, String> {
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
