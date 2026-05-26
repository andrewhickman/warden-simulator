use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_sprite::{Anchor, prelude::*};
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{
    kinematics::Position,
    tile::{
        adjacency::{TileAdjacency, WallAdjacency},
        position::TilePosition,
        storage::TileChunk,
    },
};
use wdn_world::door::Door;

use crate::{
    RenderSystems,
    assets::AssetHandles,
    layers::DOOR_LAYER,
    lerp::{FixedUpdateCount, InterpolateState},
};

pub struct DoorPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
pub struct DoorSprite {
    position: InterpolateState<f32>,
}

#[derive(Debug, Clone, Copy)]
enum DoorDirection {
    Horizontal,
    Vertical,
}

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, RenderSystems::RenderDoors);

        app.register_required_components::<Door, DoorSprite>();

        app.add_systems(Update, update_doors.in_set(RenderSystems::RenderDoors));
    }
}

pub fn update_doors(
    updates: Res<FixedUpdateCount>,
    handles: Res<AssetHandles>,
    mut doors: Query<
        (
            &Door,
            &TilePosition,
            Ref<TileAdjacency>,
            &mut DoorSprite,
            &mut Transform,
            &mut Sprite,
            &mut Anchor,
        ),
        (Without<Position>, Without<TileChunk>),
    >,
    time: Res<Time<Fixed>>,
) {
    let overstep = time.overstep_fraction();

    doors.par_iter_mut().for_each(
        |(door, tile, adjacency, mut state, mut transform, mut sprite, mut anchor)| {
            let direction = door_direction(&adjacency);
            if adjacency.is_changed() {
                match direction {
                    DoorDirection::Horizontal => {
                        *sprite = handles.door_horizontal();
                        *anchor = Anchor::BOTTOM_LEFT;
                        transform.translation =
                            Vec3::new(tile.x() as f32, tile.y() as f32 + 0.12, DOOR_LAYER);
                    }
                    DoorDirection::Vertical => {
                        *sprite = handles.door_vertical();
                        *anchor = Anchor::BOTTOM_CENTER;
                        transform.translation =
                            Vec3::new(tile.x() as f32 + 0.5, tile.y() as f32 - 0.4, DOOR_LAYER);
                    }
                }
            }

            if let Some(position) =
                state
                    .position
                    .interpolate(door.position(), overstep, updates.updated())
            {
                match direction {
                    DoorDirection::Horizontal => {
                        transform.translation.x = tile.x() as f32 - position;
                    }
                    DoorDirection::Vertical => {
                        transform.translation.y = tile.y() as f32 + position - 0.4;
                    }
                }
            }
        },
    );
}

fn door_direction(adjacency: &TileAdjacency) -> DoorDirection {
    let walls = adjacency.walls();
    if walls.contains(WallAdjacency::WEST | WallAdjacency::EAST) {
        DoorDirection::Horizontal
    } else if walls.contains(WallAdjacency::NORTH | WallAdjacency::SOUTH) {
        DoorDirection::Vertical
    } else if walls.intersects(WallAdjacency::WEST | WallAdjacency::EAST) {
        DoorDirection::Horizontal
    } else if walls.intersects(WallAdjacency::NORTH | WallAdjacency::SOUTH) {
        DoorDirection::Vertical
    } else {
        DoorDirection::Horizontal
    }
}
