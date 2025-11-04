pub mod lerp;
pub mod tile;

use bevy::prelude::*;

use crate::{lerp::InterpolatePlugin, tile::TilePlugin};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InterpolatePlugin, TilePlugin));
    }
}
