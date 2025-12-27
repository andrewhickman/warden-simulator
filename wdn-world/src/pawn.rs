use bevy_ecs::prelude::*;
use wdn_physics::{collision::Collider, integrate::Velocity, lerp::Interpolated};

use crate::health::Health;

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
