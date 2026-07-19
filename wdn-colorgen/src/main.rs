use bevy_color::{Mix, Oklaba, Srgba};

pub const SHADOW: Srgba = Srgba::rgb(0.1608, 0.2235, 0.2627);
pub const HIGHLIGHT: Srgba = Srgba::rgb(0.9020, 0.9333, 0.8824);

pub const SHADOW2: Srgba = Srgba::rgb(0.2039, 0.2745, 0.3255);
pub const HIGHLIGHT2: Srgba = Srgba::rgb(0.9490, 0.8745, 0.7529);

pub const WALL: Srgba = Srgba::rgb(0.4627, 0.3137, 0.2471);
pub const MORTAR: Srgba = Srgba::rgb(0.7176, 0.6627, 0.5843);
pub const FLOOR: Srgba = Srgba::rgb(0.4353, 0.4627, 0.4706);
pub const DIRT: Srgba = Srgba::rgb(0.2980, 0.2196, 0.1765);
pub const GRASS: Srgba = Srgba::rgb(0.4000, 0.4588, 0.3216);

pub const METAL: Srgba = Srgba::rgb(0.4078, 0.4588, 0.4863);
pub const WOOD: Srgba = Srgba::rgb(0.5176, 0.3608, 0.2471);
pub const SHEET: Srgba = Srgba::rgb(0.5686, 0.6784, 0.7216);

pub const ACCENT: Srgba = Srgba::rgb(0.0784, 0.4627, 0.5059);

pub fn shade(base: Srgba, amount: f32) -> Srgba {
    let amount = amount.clamp(0.0, 1.0);
    let base_oklab: Oklaba = base.into();

    let target = SHADOW.into();

    base_oklab.mix(&target, amount).into()
}

pub fn highlight(base: Srgba, amount: f32) -> Srgba {
    let amount = amount.clamp(0.0, 1.0);
    let base_oklab: Oklaba = base.into();

    let target = HIGHLIGHT.into();

    base_oklab.mix(&target, amount).into()
}

fn print_shades(base: Srgba) {
    println!(
        "top: {}, front: {}, side: {}, occluded: {}",
        base.to_hex(),
        shade(base, 0.33).to_hex(),
        shade(base, 0.66).to_hex(),
        shade(base, 0.88).to_hex()
    );
}

fn print_shade_and_highlight(base: Srgba) {
    println!(
        "base: {}, shade: {}, {}, highlight: {}, {}",
        base.to_hex(),
        shade(base, 0.1).to_hex(),
        shade(base, 0.2).to_hex(),
        highlight(base, 0.1).to_hex(),
        highlight(base, 0.2).to_hex(),
    );
}

fn main() {
    println!("ACCENT");
    print_shade_and_highlight(ACCENT);
}
