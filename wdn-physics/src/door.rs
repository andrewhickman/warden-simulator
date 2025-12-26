use bevy_ecs::prelude::*;

use crate::collision::TileCollider;

#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TileCollider)]
pub struct Door;
