use bevy_ecs::prelude::*;

use wdn_physics::collision::TileCollider;

#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TileCollider)]
pub struct Door;
