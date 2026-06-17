use bevy_ecs::{batching::BatchingStrategy, prelude::*};
use bevy_log::warn;
use wdn_physics::{
    collision::{CollisionTarget, Collisions},
    kinematics::GlobalPosition,
    tile::position::TilePosition,
};

use crate::{
    door::Door,
    path::find::{Path, PathParam},
    pawn::{Pawn, action::PawnAction},
};

#[derive(Component, Default, Debug)]
pub struct PawnPath {
    target: Option<TilePosition>,
    state: PathState,
}

#[derive(Debug, Default)]
enum PathState {
    #[default]
    Pending,
    Active(Path),
    Finished,
    Failed,
}

pub fn follow_pawn_paths(
    mut pawns: Query<(
        &mut PawnAction,
        &mut PawnPath,
        &TilePosition,
        &GlobalPosition,
    )>,
    paths: PathParam,
) {
    pawns
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(16))
        .for_each(
            |(mut action, mut pawn_path, &tile_position, &global_position)| {
                let Some(target) = pawn_path.target else {
                    return;
                };

                if tile_position == target {
                    pawn_path.state = PathState::Finished;
                    *action = PawnAction::Stand;
                    return;
                }

                let desired_dir = loop {
                    match &mut pawn_path.state {
                        PathState::Active(path) if paths.is_valid(path) => {
                            if let Some(dir) = paths.path_dir(path, tile_position) {
                                break dir;
                            } else {
                                warn!(
                                    "Failed to get path direction at position {:?}",
                                    tile_position
                                );
                                pawn_path.state = PathState::Pending;
                            }
                        }
                        PathState::Finished | PathState::Failed => {
                            return;
                        }
                        PathState::Active(_) | PathState::Pending => {
                            if let Some(new_path) = paths.find_path(tile_position, target) {
                                pawn_path.state = PathState::Active(new_path);
                            } else {
                                warn!(
                                    "Failed to find path from {:?} to {:?}",
                                    tile_position, target
                                );
                                pawn_path.state = PathState::Failed;
                            }
                        }
                    };
                };

                let actual_dir = global_position.rotation();
                let delta = actual_dir.angle_to(desired_dir.rotation_from_x());

                if delta.abs() > 1.0 {
                    *action = if delta > 0.0 {
                        PawnAction::TurnLeft
                    } else {
                        PawnAction::TurnRight
                    };
                } else if delta.abs() > 0.1 {
                    *action = if delta > 0.0 {
                        PawnAction::SteerLeft
                    } else {
                        PawnAction::SteerRight
                    };
                } else {
                    *action = PawnAction::Walk;
                }
            },
        );
}

pub fn open_doors_on_collision(
    collisions: Query<&Collisions, With<Pawn>>,
    mut doors: Query<&mut Door>,
) {
    collisions.iter().for_each(|collisions| {
        for collision in collisions.iter() {
            if !collision.solid {
                continue;
            }

            match collision.target {
                CollisionTarget::Tile {
                    id: Some(tile_id), ..
                } => {
                    if let Ok(mut door) = doors.get_mut(tile_id) {
                        door.open();
                    }
                }
                _ => {}
            }
        }
    });
}

impl PawnPath {
    pub fn set_target(&mut self, target: TilePosition) {
        self.target = Some(target);
        self.state = PathState::Pending;
    }

    pub fn target(&self) -> Option<TilePosition> {
        self.target
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.state {
            PathState::Active(path) => Some(path),
            _ => None,
        }
    }
}
