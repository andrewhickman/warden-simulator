use std::{f32::consts::TAU, time::Duration};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::Vec2;
use bevy_time::Time;
use bevy_transform::prelude::*;
use wdn_physics::{
    collision::Collider,
    kinematics::{Position, Velocity},
    lerp::Interpolate,
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
    Interpolate,
    Health::new(Pawn::MAX_HEALTH),
    PawnAction
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
    Interpolate,
    Projectile::new(Entity::PLACEHOLDER, PawnProjectile::DAMAGE, PawnProjectile::DURATION)
)]
pub struct PawnProjectile;

#[derive(Copy, Clone, Component, Debug, Default)]
pub enum PawnAction {
    #[default]
    Stand,
    Walk,
    TurnLeft,
    TurnRight,
    SteerLeft,
    SteerRight,
    AttackLeft,
    AttackRight,
}

pub fn apply_pawn_actions(
    commands: ParallelCommands,
    mut query: Query<(Entity, &Position, &mut Velocity, &PawnAction), With<Pawn>>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .for_each(|(id, position, mut velocity, action)| match action {
            PawnAction::Stand => {
                velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                velocity.set_angular(0.0);
            }
            PawnAction::Walk => {
                velocity.accelerate(
                    position.rotation() * Vec2::new(0.0, Pawn::WALK_SPEED),
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
            PawnAction::SteerLeft => {
                velocity.accelerate(
                    position.rotation() * Vec2::new(0.0, Pawn::WALK_SPEED * 0.75),
                    Pawn::ACCELERATION * 0.75 * time.delta_secs(),
                );
                velocity.set_angular(Pawn::TURN_SPEED * 0.7);
            }
            PawnAction::SteerRight => {
                velocity.accelerate(
                    position.rotation() * Vec2::new(0.0, Pawn::WALK_SPEED * 0.75),
                    Pawn::ACCELERATION * 0.75 * time.delta_secs(),
                );
                velocity.set_angular(-Pawn::TURN_SPEED * 0.7);
            }
            PawnAction::AttackLeft => commands.command_scope(|mut commands| {
                commands.spawn(PawnProjectile::new(
                    id,
                    Vec2::new(-PawnProjectile::OFFSET, 0.0),
                    Vec2::new(0.0, PawnProjectile::SPEED),
                ));
            }),
            PawnAction::AttackRight => commands.command_scope(|mut commands| {
                commands.spawn(PawnProjectile::new(
                    id,
                    Vec2::new(PawnProjectile::OFFSET, 0.0),
                    Vec2::new(0.0, PawnProjectile::SPEED),
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

    pub fn new(pawn: Entity, position: Vec2, velocity: Vec2) -> impl Bundle {
        (
            PawnProjectile,
            Projectile::new(pawn, PawnProjectile::DAMAGE, PawnProjectile::DURATION),
            ChildOf(pawn),
            Transform::from_translation(position.extend(1.0)),
            Velocity::new(velocity),
        )
    }
}
