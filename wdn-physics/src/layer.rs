use bevy_ecs::prelude::*;
use bevy_transform::prelude::*;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Transform)]
pub struct Layer {}
