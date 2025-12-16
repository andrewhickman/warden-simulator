pub mod collision;
pub mod integrate;
pub mod lerp;
pub mod tile;

use bevy::prelude::*;

use crate::{
    collision::CollisionPlugin, integrate::IntegratePlugin, lerp::InterpolatePlugin,
    tile::TilePlugin,
};

pub struct PhysicsPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum PhysicsSystems {
    UpdateTile,
    ResolveCollisions,
    Integrate,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InterpolatePlugin,
            TilePlugin,
            CollisionPlugin,
            IntegratePlugin,
        ));
    }
}
