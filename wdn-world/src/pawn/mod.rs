pub mod action;
pub mod path;

use std::{f32::consts::TAU, time::Duration};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::{Rot2, Vec2};
use bevy_transform::prelude::*;
use wdn_physics::{
    PhysicsSystems,
    collision::Collider,
    kinematics::{Position, Velocity},
    tile::material::TileMaterial,
};

use crate::{
    WorldSystems,
    combat::{Health, Projectile},
    pawn::{
        action::{PawnAction, apply_pawn_actions},
        path::{PawnPath, follow_pawn_paths, open_doors_on_collision},
    },
};

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::RADIUS, true),
    Transform,
    Velocity,
    Health::new(Pawn::MAX_HEALTH),
    PawnAction,
    PawnPath,
    TileMaterial
)]
#[expect(unused)]
pub struct Pawn {
    health: u32,
    stamina: u32,
    left_attack_cooldown: Duration,
    right_attack_cooldown: Duration,
}

#[derive(Copy, Clone, Component, Debug)]
#[require(
    Collider::new(PawnProjectile::RADIUS, false),
    Transform,
    Projectile::new(Entity::PLACEHOLDER, PawnProjectile::DAMAGE, PawnProjectile::DURATION)
)]
pub struct PawnProjectile;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            WorldSystems::ApplyPawnActions
                .after(WorldSystems::UpdateRegions)
                .before(WorldSystems::ApplyProjectiles)
                .before(PhysicsSystems::Kinematics),
        );

        app.add_systems(
            FixedUpdate,
            (
                (follow_pawn_paths, apply_pawn_actions)
                    .chain()
                    .in_set(WorldSystems::ApplyPawnActions),
                (open_doors_on_collision.after(PhysicsSystems::Collisions)),
            ),
        );
    }
}

impl Pawn {
    pub const RADIUS: f32 = 0.2;
    pub const MAX_HEALTH: u32 = 5;
    pub const WALK_SPEED: f32 = 1.5;
    pub const TURN_SPEED: f32 = TAU;
    pub const ACCELERATION: f32 = 6.0;
}

impl PawnProjectile {
    pub const OFFSET: f32 = 0.12;
    pub const RADIUS: f32 = 0.08;
    pub const DAMAGE: u32 = 1;
    pub const DURATION: Duration = Duration::from_millis(500);
    pub const SPEED: f32 = 0.86;

    pub fn bundle(pawn: Entity, position: Vec2, velocity: Vec2) -> impl Bundle {
        (
            PawnProjectile,
            Projectile::new(pawn, PawnProjectile::DAMAGE, PawnProjectile::DURATION),
            ChildOf(pawn),
            Position::new(position, Rot2::IDENTITY),
            Velocity::new(velocity),
        )
    }
}
