use bevy_color::{Mix, Oklaba, Srgba};

pub const SHADOW: Oklaba = Oklaba::new(0.18, -0.025, -0.035, 1.0);

pub const WALL: Srgba = Srgba::rgb(0.44313726, 0.41960785, 0.38039216);
pub const FLOOR: Srgba = Srgba::rgb(0.654902, 0.6666667, 0.6392157);
pub const DIRT: Srgba = Srgba::rgb(0.38431373, 0.36078432, 0.3254902);
pub const GRASS: Srgba = Srgba::rgb(0.37254903, 0.4627451, 0.3137255);

pub fn shade(base: Srgba, amount: f32) -> Srgba {
    let amount = amount.clamp(0.0, 1.0);
    let base_oklab: Oklaba = base.into();

    let target = Oklaba::new(SHADOW.lightness, SHADOW.a, SHADOW.b, base_oklab.alpha);

    base_oklab.mix(&target, amount).into()
}

fn main() {
    println!(
        "WALL: top: {}, front: {}, side: {}, occluded: {}",
        WALL.to_hex(),
        shade(WALL, 0.1).to_hex(),
        shade(WALL, 0.35).to_hex(),
        shade(WALL, 0.65).to_hex()
    );
    println!(
        "FLOOR: top: {}, front: {}, side: {}",
        FLOOR.to_hex(),
        shade(FLOOR, 0.35).to_hex(),
        shade(FLOOR, 0.65).to_hex(),
    );
}
