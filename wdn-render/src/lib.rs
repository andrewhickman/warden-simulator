pub mod assets;
pub mod damage;
pub mod pawn;
pub mod tile;

use bevy_app::prelude::*;

use crate::{assets::AssetsPlugin, damage::DamagePlugin, pawn::PawnPlugin, tile::TilePlugin};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AssetsPlugin, DamagePlugin, TilePlugin, PawnPlugin));
    }
}
