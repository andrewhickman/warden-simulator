pub mod assets;
pub mod pawn;
pub mod tile;

use bevy_app::prelude::*;

use crate::{assets::AssetsPlugin, pawn::PawnPlugin, tile::TilePlugin};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AssetsPlugin, TilePlugin, PawnPlugin));
    }
}
