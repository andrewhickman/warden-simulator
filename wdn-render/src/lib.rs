pub mod assets;
pub mod damage;
pub mod lerp;
pub mod pawn;
pub mod tile;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    assets::AssetsPlugin, damage::DamagePlugin, lerp::InterpolatePlugin, pawn::PawnPlugin,
    tile::TilePlugin,
};

pub struct RenderPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum RenderSystems {
    Interpolate,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AssetsPlugin,
            DamagePlugin,
            TilePlugin,
            PawnPlugin,
            InterpolatePlugin,
        ));
    }
}
