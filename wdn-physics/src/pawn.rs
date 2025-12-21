use bevy::prelude::*;

use crate::{collision::Collider, integrate::Velocity, lerp::Interpolated};

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Collider::new(Pawn::RADIUS), Velocity, Interpolated)]
pub struct Pawn;

impl Pawn {
    pub const RADIUS: f32 = 0.24;
}
