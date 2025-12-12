pub mod assets;
pub mod tile;

use bevy::prelude::*;

use crate::{assets::AssetsPlugin, tile::TilePlugin};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AssetsPlugin, TilePlugin));
    }
}
