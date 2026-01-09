pub mod collision;
pub mod kinematics;
pub mod layer;
pub mod lerp;
pub mod sync;
pub mod tile;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    collision::CollisionPlugin, kinematics::KinematicsPlugin, lerp::InterpolatePlugin,
    sync::SyncPlugin, tile::TilePlugin,
};

pub struct PhysicsPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum PhysicsSystems {
    Sync,
    Collisions,
    Kinematics,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TilePlugin,
            InterpolatePlugin,
            SyncPlugin,
            CollisionPlugin,
            KinematicsPlugin,
        ));
    }
}
