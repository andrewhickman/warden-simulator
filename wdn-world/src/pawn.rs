use std::{f32::consts::TAU, time::Duration};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::{Dir2, Vec2};
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{collision::Collider, integrate::Velocity, lerp::Interpolated};

use crate::{
    WorldSystems,
    combat::{Health, Projectile},
};

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::RADIUS, true),
    Transform,
    Velocity,
    Interpolated,
    Health::new(Pawn::MAX_HEALTH),
    PawnAction
)]
pub struct Pawn {
    rotation: u8,
}

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(Pawn::PROJECTILE_RADIUS, false),
    Transform,
    Interpolated,
    Projectile::new(PawnProjectile::DAMAGE, PawnProjectile::DURATION)
)]
pub struct PawnProjectile;

#[derive(Copy, Clone, Component, Debug, Default)]
pub enum PawnAction {
    #[default]
    Stand,
    Walk,
    TurnLeft,
    TurnRight,
    AttackLeft,
    AttackRight,
}

pub fn apply_pawn_actions(
    commands: ParallelCommands,
    mut query: Query<(Entity, &mut Pawn, &mut Velocity, &PawnAction)>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(id, mut pawn, mut velocity, action)| match action {
            PawnAction::Stand => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
            }
            PawnAction::Walk => {
                velocity.accelerate(
                    pawn.direction() * Pawn::WALK_SPEED,
                    Pawn::ACCELERATION * time.delta_secs(),
                );
            }
            PawnAction::TurnLeft => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                pawn.rotation = pawn.rotation.wrapping_add(1) % Pawn::ROTATION_INCREMENTS;
            }
            PawnAction::TurnRight => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                pawn.rotation = pawn.rotation.wrapping_sub(1) % Pawn::ROTATION_INCREMENTS;
            }
            PawnAction::AttackLeft => commands.command_scope(|mut commands| {
                commands.spawn((
                    PawnProjectile,
                    ChildOf(id),
                    Transform::from_xyz(-Pawn::RADIUS, 0.0, 0.0),
                    Velocity::new(Vec2::new(0.0, PawnProjectile::SPEED)),
                ));
            }),
            PawnAction::AttackRight => commands.command_scope(|mut commands| {
                commands.spawn((
                    PawnProjectile,
                    ChildOf(id),
                    Transform::from_xyz(Pawn::RADIUS, 0.0, 0.0),
                    Velocity::new(Vec2::new(0.0, PawnProjectile::SPEED)),
                ));
            }),
        });
}

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            WorldSystems::ApplyPawnActions.before(WorldSystems::ApplyProjectiles),
        );

        app.add_systems(
            FixedUpdate,
            apply_pawn_actions.in_set(WorldSystems::ApplyPawnActions),
        );
    }
}

impl Pawn {
    pub const RADIUS: f32 = 0.24;
    pub const PROJECTILE_RADIUS: f32 = 0.08;
    pub const MAX_HEALTH: u32 = 100;
    pub const WALK_SPEED: f32 = 1.5;
    pub const ACCELERATION: f32 = 6.0;
    pub const ROTATION_INCREMENTS: u8 = 64;

    pub fn rotation(&self) -> f32 {
        self.rotation as f32 * (TAU / Pawn::ROTATION_INCREMENTS as f32)
    }

    pub fn direction(&self) -> Dir2 {
        let (sin, cos) = self.rotation().sin_cos();
        Dir2::from_xy_unchecked(cos, sin)
    }
}

impl PawnProjectile {
    pub const DAMAGE: u32 = 1;
    pub const DURATION: Duration = Duration::from_secs(1);
    pub const SPEED: f32 = 0.86;
}
