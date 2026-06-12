use bevy_ecs::{batching::BatchingStrategy, prelude::*};
use bevy_log::{info, warn};
use wdn_physics::{kinematics::GlobalPosition, tile::position::TilePosition};

use crate::{
    path::find::{Path, PathParam},
    pawn::action::PawnAction,
};

#[derive(Component, Default)]
pub struct PawnPath {
    target: Option<TilePosition>,
    path: Option<Path>,
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
                    pawn_path.path = None;
                    *action = PawnAction::Stand;
                    return;
                }

                let path = match pawn_path.path.as_mut() {
                    Some(path) if paths.is_valid(path) => path,
                    _ => {
                        let Some(new_path) = paths.find_path(tile_position, target) else {
                            warn!(
                                "Failed to find path from {:?} to {:?}",
                                tile_position, target
                            );
                            return;
                        };

                        pawn_path.path.insert(new_path)
                    }
                };

                let Some(desired_dir) = paths.path_dir(path, tile_position) else {
                    warn!(
                        "Failed to get path direction for at position {:?}",
                        tile_position
                    );
                    return;
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

impl PawnPath {
    pub fn set_target(&mut self, target: TilePosition) {
        self.target = Some(target);
        self.path = None;
    }

    pub fn target(&self) -> Option<TilePosition> {
        self.target
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }
}
