use bevy_ecs::prelude::*;

use crate::{collision::Collider, health::Health, integrate::Velocity, lerp::Interpolated};

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::RADIUS, true),
    Velocity,
    Interpolated,
    Health::new(Pawn::MAX_HEALTH)
)]
pub struct Pawn;

impl Pawn {
    pub const RADIUS: f32 = 0.24;
    pub const MAX_HEALTH: u32 = 100;
}
