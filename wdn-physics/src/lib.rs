pub mod collision;
pub mod integrate;
pub mod lerp;
pub mod tile;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    collision::CollisionPlugin, integrate::IntegratePlugin, lerp::InterpolatePlugin,
    tile::TilePlugin,
};

pub struct PhysicsPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum PhysicsSystems {
    PropagatePosition,
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
