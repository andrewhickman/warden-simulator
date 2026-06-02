pub mod assets;
pub mod damage;
pub mod dev;
pub mod door;
pub mod layers;
pub mod lerp;
pub mod pawn;
pub mod tile;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    assets::AssetsPlugin, damage::DamagePlugin, dev::DevPlugin, door::DoorPlugin,
    lerp::InterpolatePlugin, pawn::PawnPlugin, tile::TilePlugin,
};

pub struct RenderPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum RenderSystems {
    InterpolatePosition,
    RenderDoors,
    RenderDamage,
    RenderTiles,
    RenderDev,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            RenderSystems::RenderDoors.before(RenderSystems::RenderDamage),
        );

        app.add_plugins((
            AssetsPlugin,
            DamagePlugin,
            DevPlugin,
            DoorPlugin,
            TilePlugin,
            PawnPlugin,
            InterpolatePlugin,
        ));
    }
}
