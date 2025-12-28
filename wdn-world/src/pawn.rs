use bevy_ecs::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{collision::Collider, integrate::Velocity, lerp::Interpolated};

use crate::combat::{Health, Projectile};

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::RADIUS, true),
    Transform,
    Velocity,
    Interpolated,
    Health::new(Pawn::MAX_HEALTH)
)]
pub struct Pawn;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::PROJECTILE_RADIUS, false),
    Transform,
    Velocity,
    Interpolated,
    Projectile::new(1)
)]
pub struct PawnProjectile;

#[derive(Copy, Clone, Debug, Default)]
pub enum PawnAction {
    #[default]
    Stand,
    Walk,
    TurnLeft,
    TurnRight,
    AttackLeft,
    AttackRight,
}

impl Pawn {
    pub const RADIUS: f32 = 0.24;
    pub const PROJECTILE_RADIUS: f32 = 0.08;
    pub const MAX_HEALTH: u32 = 100;
}
