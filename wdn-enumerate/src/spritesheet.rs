use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use roxmltree::{Document, Node};
use wdn_enumerate::{BaseSprite, OverlapBase, TopSprite};

const TILE_WIDTH: usize = 200;
const TILE_HEIGHT: usize = 400;

/// Builds `spritesheet.svg` from `assets/image/walls.svg`, placing each base and top sprite
/// group at the row-major grid cell given by its `id()`.
pub fn generate(ordered_base_sprites: &[BaseSprite], ordered_top_sprites: &[TopSprite]) {
    let source = std::fs::read_to_string("assets/image/walls.svg")
        .expect("failed to read assets/image/walls.svg");
    let doc = Document::parse(&source).expect("failed to parse assets/image/walls.svg");

    let mut groups_by_label: HashMap<&str, Node> = HashMap::new();
    for node in doc.descendants() {
        if node.has_tag_name("g") {
            if let Some(label) = node_label(node) {
                groups_by_label.entry(label).or_insert(node);
            }
        }
    }

    let max_id = ordered_base_sprites
        .iter()
        .map(|base| base.id() as usize)
        .chain(ordered_top_sprites.iter().map(|top| top.id()))
        .max()
        .unwrap_or(0);
    let rows = max_id / 16 + 1;
    let height = rows * TILE_HEIGHT;

    let mut svg =
        BufWriter::new(File::create("spritesheet.svg").expect("failed to create spritesheet.svg"));
    writeln!(
        svg,
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#
    )
    .unwrap();
    writeln!(
        svg,
        r#"<svg width="3200" height="{height}" viewBox="0 0 3200 {height}" version="1.1" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns="http://www.w3.org/2000/svg">"#
    )
    .unwrap();

    let mut group_bufs: Vec<String> = Vec::new();

    for base in ordered_base_sprites {
        if *base == BaseSprite::Empty {
            continue;
        }
        let label = base.to_string();
        if let Some(buf) =
            write_sprite_group(&groups_by_label, &label, &label, base.id() as usize, None)
        {
            group_bufs.push(buf);
        }
    }

    for top in ordered_top_sprites {
        if *top == TopSprite::EMPTY {
            continue;
        }
        let label = top.to_string();
        let core_label = top.without_overlap().to_string();
        let overlap = top.overlap();
        let patch_label = (overlap != OverlapBase::None).then(|| overlap.base_sprite().to_string());
        if let Some(buf) = write_sprite_group(
            &groups_by_label,
            &label,
            &core_label,
            top.id(),
            patch_label.as_deref(),
        ) {
            group_bufs.push(buf);
        }
    }

    group_bufs.reverse();

    for buf in &group_bufs {
        svg.write_all(buf.as_bytes()).unwrap();
    }

    writeln!(svg, "</svg>").unwrap();
    svg.flush().unwrap();
    println!("wrote spritesheet.svg ({rows} rows, {height}px tall)");

    // rasterize for a quick visual sanity check
    let svg_text = std::fs::read_to_string("spritesheet.svg").unwrap();
    let tree = resvg::usvg::Tree::from_str(&svg_text, &resvg::usvg::Options::default()).unwrap();
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png("spritesheet.png").unwrap();
}

/// Writes one sprite's group: its own shape copied from `source_label`'s group, plus (if
/// `patch_label` is given) the "Top" path borrowed from that base sprite's group.
fn write_sprite_group(
    groups_by_label: &HashMap<&str, Node>,
    output_label: &str,
    source_label: &str,
    id: usize,
    patch_label: Option<&str>,
) -> Option<String> {
    let col = id % 16;
    let row = id / 16;
    let x = col * TILE_WIDTH;
    let y = (row + 1) * TILE_HEIGHT;

    let Some(&source) = groups_by_label.get(source_label) else {
        eprintln!("warning: no SVG group found for {source_label}, skipping");
        return None;
    };

    let mut buf: Vec<u8> = Vec::new();
    writeln!(
        buf,
        r#"  <g inkscape:label="{output_label}" transform="translate({x},{y})">"#
    )
    .unwrap();
    write_wrapped(&mut buf, source.attribute("transform"), 4, |out, indent| {
        for child in source.children().filter(Node::is_element) {
            write_element(out, child, indent);
        }
    });

    if let Some(patch_label) = patch_label {
        if let Some(&patch_source) = groups_by_label.get(patch_label) {
            if let Some(top_path) = patch_source
                .children()
                .find(|c| node_label(*c) == Some("Top"))
            {
                write_wrapped(
                    &mut buf,
                    patch_source.attribute("transform"),
                    4,
                    |out, indent| {
                        write_element(out, top_path, indent);
                    },
                );
            } else {
                eprintln!(
                    "warning: {patch_label} has no \"Top\" path to patch into {output_label}"
                );
            }
        } else {
            eprintln!(
                "warning: no SVG group found for {patch_label} (overlap base of {output_label})"
            );
        }
    }
    writeln!(buf, "  </g>").unwrap();

    Some(String::from_utf8(buf).unwrap())
}

/// Wraps `content` in a `<g transform="...">` matching the source group's original transform,
/// so copied children render exactly as they did in their original tile-local position.
fn write_wrapped(
    out: &mut dyn Write,
    transform: Option<&str>,
    indent: usize,
    content: impl FnOnce(&mut dyn Write, usize),
) {
    match transform {
        Some(transform) => writeln!(out, "{:indent$}<g transform=\"{transform}\">", "").unwrap(),
        None => writeln!(out, "{:indent$}<g>", "").unwrap(),
    }
    content(out, indent + 2);
    writeln!(out, "{:indent$}</g>", "").unwrap();
}

/// Recursively copies an element, keeping only plain (non-namespaced) SVG attributes.
fn write_element(out: &mut dyn Write, node: Node, indent: usize) {
    let tag = node.tag_name().name();
    write!(out, "{:indent$}<{tag}", "").unwrap();
    for attr in node.attributes() {
        if attr.namespace().is_some() {
            continue;
        }
        write!(out, " {}=\"{}\"", attr.name(), escape_xml(attr.value())).unwrap();
    }
    let children: Vec<Node> = node.children().filter(Node::is_element).collect();
    if children.is_empty() {
        writeln!(out, " />").unwrap();
    } else {
        writeln!(out, ">").unwrap();
        for child in children {
            write_element(out, child, indent + 2);
        }
        writeln!(out, "{:indent$}</{tag}>", "").unwrap();
    }
}

fn node_label<'a>(node: Node<'a, 'a>) -> Option<&'a str> {
    node.attributes()
        .find(|attr| attr.name() == "label")
        .map(|attr| attr.value())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
