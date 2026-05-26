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
    assets::{AssetHandles, DOOR_HORIZONTAL_RECT, DOOR_VERTICAL_RECT, sprite_size},
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

    let vertical_door_size = sprite_size(DOOR_VERTICAL_RECT);
    let horizontal_door_size = sprite_size(DOOR_HORIZONTAL_RECT);

    doors.par_iter_mut().for_each(
        |(door, tile, adjacency, mut state, mut transform, mut sprite, mut anchor)| {
            let direction = DoorDirection::from_adjacency(&adjacency);
            if adjacency.is_changed() {
                *sprite = direction.sprite(&handles);
                *anchor = direction.anchor();
                state.position.reset();
            }

            if let Some(position) =
                state
                    .position
                    .interpolate(door.position(), overstep, updates.updated())
            {
                transform.translation = direction.translation(*tile, position);
            }
        },
    );
}

impl DoorDirection {
    fn from_adjacency(adjacency: &TileAdjacency) -> Self {
        let walls = adjacency.walls();
        if walls.contains(WallAdjacency::WEST | WallAdjacency::EAST) {
            Self::Horizontal
        } else if walls.contains(WallAdjacency::NORTH | WallAdjacency::SOUTH) {
            Self::Vertical
        } else if walls.intersects(WallAdjacency::WEST | WallAdjacency::EAST) {
            Self::Horizontal
        } else if walls.intersects(WallAdjacency::NORTH | WallAdjacency::SOUTH) {
            Self::Vertical
        } else {
            Self::Horizontal
        }
    }

    fn sprite(&self, handles: &AssetHandles) -> Sprite {
        match self {
            DoorDirection::Horizontal => handles.door_horizontal(),
            DoorDirection::Vertical => handles.door_vertical(),
        }
    }

    fn anchor(&self) -> Anchor {
        match self {
            DoorDirection::Horizontal => Anchor::BOTTOM_LEFT,
            DoorDirection::Vertical => Anchor::CENTER,
        }
    }

    fn translation(&self, tile: TilePosition, position: f32) -> Vec3 {
        match self {
            DoorDirection::Horizontal => Vec3::new(
                tile.x() as f32 - position,
                tile.y() as f32 + 0.12,
                DOOR_LAYER,
            ),
            DoorDirection::Vertical => Vec3::new(
                tile.x() as f32 + 0.5,
                tile.y() as f32 + f32::lerp(0.579, 1.85, position),
                DOOR_LAYER,
            ),
        }
    }
}
