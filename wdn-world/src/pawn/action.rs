use bevy_ecs::{batching::BatchingStrategy, prelude::*};
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use wdn_physics::{
    kinematics::{Position, Velocity},
    tile::material::TileMaterial,
};

use crate::pawn::{Pawn, PawnProjectile};

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
    mut query: Query<(Entity, &Position, &mut Velocity, &TileMaterial, &PawnAction), With<Pawn>>,
    time: Res<Time>,
) {
    query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(16))
        .for_each(
            |(id, position, mut velocity, tile_material, action)| match action {
                PawnAction::Stand => {
                    velocity.decelerate(Pawn::ACCELERATION * time.delta_secs());
                    velocity.set_angular(0.0);
                }
                PawnAction::Walk => {
                    velocity.accelerate(
                        position.rotation()
                            * Vec2::new(
                                Pawn::WALK_SPEED * tile_material.move_speed().factor(),
                                0.0,
                            ),
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
                        position.rotation()
                            * Vec2::new(
                                Pawn::WALK_SPEED * 0.75 * tile_material.move_speed().factor(),
                                0.0,
                            ),
                        Pawn::ACCELERATION * 0.75 * time.delta_secs(),
                    );
                    velocity.set_angular(Pawn::TURN_SPEED * 0.7);
                }
                PawnAction::SteerRight => {
                    velocity.accelerate(
                        position.rotation()
                            * Vec2::new(
                                Pawn::WALK_SPEED * 0.75 * tile_material.move_speed().factor(),
                                0.0,
                            ),
                        Pawn::ACCELERATION * 0.75 * time.delta_secs(),
                    );
                    velocity.set_angular(-Pawn::TURN_SPEED * 0.7);
                }
                PawnAction::AttackLeft => commands.command_scope(|mut commands| {
                    commands.spawn(PawnProjectile::bundle(
                        id,
                        Vec2::new(-PawnProjectile::OFFSET, 0.0),
                        Vec2::new(0.0, PawnProjectile::SPEED),
                    ));
                }),
                PawnAction::AttackRight => commands.command_scope(|mut commands| {
                    commands.spawn(PawnProjectile::bundle(
                        id,
                        Vec2::new(PawnProjectile::OFFSET, 0.0),
                        Vec2::new(0.0, PawnProjectile::SPEED),
                    ));
                }),
            },
        );
}
