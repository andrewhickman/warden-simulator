pub mod collision;
pub mod kinematics;
pub mod layer;
pub mod lerp;
pub mod tile;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    collision::CollisionPlugin, kinematics::KinematicsPlugin, lerp::InterpolatePlugin,
    tile::TilePlugin,
};

pub struct PhysicsPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum PhysicsSystems {
    Collisions,
    Kinematics,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TilePlugin,
            InterpolatePlugin,
            CollisionPlugin,
            KinematicsPlugin,
        ));
    }
}
