use std::{f32::consts::TAU, time::Duration};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::Vec2;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{
    collision::Collider, integrate::Velocity, lerp::Interpolated, transform::quat_to_rot,
};

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
pub struct Pawn;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(
    Collider::new(PawnProjectile::RADIUS, false),
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
    mut query: Query<(Entity, &Transform, &mut Velocity, &PawnAction), With<Pawn>>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(id, transform, mut velocity, action)| match action {
            PawnAction::Stand => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                velocity.set_angular(0.0);
            }
            PawnAction::Walk => {
                velocity.accelerate(
                    quat_to_rot(transform.rotation) * Vec2::new(0.0, Pawn::WALK_SPEED),
                    Pawn::ACCELERATION * time.delta_secs(),
                );
                velocity.set_angular(0.0);
            }
            PawnAction::TurnLeft => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                velocity.set_angular(Pawn::TURN_SPEED);
            }
            PawnAction::TurnRight => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                velocity.set_angular(-Pawn::TURN_SPEED);
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
    pub const MAX_HEALTH: u32 = 100;
    pub const WALK_SPEED: f32 = 1.5;
    pub const TURN_SPEED: f32 = TAU;
    pub const ACCELERATION: f32 = 6.0;
}

impl PawnProjectile {
    pub const RADIUS: f32 = 0.08;
    pub const DAMAGE: u32 = 1;
    pub const DURATION: Duration = Duration::from_secs(1);
    pub const SPEED: f32 = 0.86;
}
