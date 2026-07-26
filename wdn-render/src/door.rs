use bevy_app::prelude::*;
use bevy_ecs::{batching::BatchingStrategy, prelude::*};
use bevy_math::prelude::*;
use bevy_sprite::{Anchor, prelude::*};
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{
    kinematics::Position,
    tile::{
        adjacency::{Adjacency, TileAdjacency},
        position::TilePosition,
        storage::TileChunk,
    },
};
use wdn_world::door::{Door, DoorDirection};

use crate::{
    RenderSystems,
    assets::{
        AssetHandles, DOOR_HORIZONTAL_RECT, DOOR_VERTICAL_RECT, SPRITE_SCALE_FACTOR_RECIP,
        sprite_size,
    },
    depth::SPRITE_DEPTH,
    lerp::{FixedUpdateCount, InterpolateState},
};

pub struct DoorPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite, Anchor::CENTER)]
pub struct DoorSprite {
    position: InterpolateState<f32>,
}

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            RenderSystems::RenderDoors.before(RenderSystems::RenderDamage),
        );

        app.register_required_components::<Door, DoorSprite>();

        app.add_systems(Update, update_doors.in_set(RenderSystems::RenderDoors));
    }
}

pub fn update_doors(
    updates: Res<FixedUpdateCount>,
    handles: Res<AssetHandles>,
    mut doors: Query<
        (
            &TilePosition,
            &Door,
            Ref<DoorDirection>,
            &TileAdjacency,
            &mut DoorSprite,
            &mut Transform,
            &mut Sprite,
        ),
        (Without<Position>, Without<TileChunk>),
    >,
    time: Res<Time<Fixed>>,
) {
    let overstep = time.overstep_fraction();

    doors
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(16))
        .for_each(
            |(tile, door, direction, adjacency, mut state, mut transform, mut sprite)| {
                if direction.is_changed() {
                    *sprite = door_sprite(*direction, &handles);
                    state.position.reset();
                }

                if let Some(position) =
                    state
                        .position
                        .interpolate(door.position(), overstep, updates.updated())
                {
                    let translation = door_translation(*direction, *tile, position);
                    let sprite_size = door_sprite_size(*direction);

                    let sprite_rect = Rect::from_center_size(translation, sprite_size);
                    let clip_rect = door_clip_rect(*direction, *tile, adjacency.walls());

                    let clipped_rect = sprite_rect.intersect(clip_rect);
                    if clipped_rect.is_empty() {
                        sprite.rect = Some(Rect::EMPTY);
                        sprite.custom_size = Some(Vec2::ZERO);
                        return;
                    }

                    let size = clipped_rect.size();
                    let offset = Vec2::new(
                        clipped_rect.min.x - sprite_rect.min.x,
                        sprite_rect.max.y - clipped_rect.max.y,
                    );

                    transform.translation = clipped_rect.center().extend(SPRITE_DEPTH);
                    sprite.rect = Some(Rect::from_corners(
                        offset * SPRITE_SCALE_FACTOR_RECIP,
                        (offset + size) * SPRITE_SCALE_FACTOR_RECIP,
                    ));
                    sprite.custom_size = Some(size);
                }
            },
        );
}

fn door_sprite(direction: DoorDirection, handles: &AssetHandles) -> Sprite {
    match direction {
        DoorDirection::Horizontal => handles.door_horizontal(),
        DoorDirection::Vertical => handles.door_vertical(),
    }
}

fn door_sprite_size(direction: DoorDirection) -> Vec2 {
    match direction {
        DoorDirection::Horizontal => sprite_size(DOOR_HORIZONTAL_RECT),
        DoorDirection::Vertical => sprite_size(DOOR_VERTICAL_RECT),
    }
}

fn door_translation(direction: DoorDirection, tile: TilePosition, position: f32) -> Vec2 {
    match direction {
        DoorDirection::Horizontal => {
            Vec2::new(tile.x() as f32 + 0.5 - position, tile.y() as f32 + 0.56)
        }
        DoorDirection::Vertical => Vec2::new(
            tile.x() as f32 + 0.5,
            tile.y() as f32 + f32::lerp(0.579, 1.85, position),
        ),
    }
}

fn door_clip_rect(direction: DoorDirection, tile: TilePosition, walls: Adjacency) -> Rect {
    match direction {
        DoorDirection::Horizontal => Rect::new(
            if walls.contains(Adjacency::WEST) {
                tile.x() as f32
            } else {
                tile.x() as f32 - 1.0
            },
            tile.y() as f32,
            tile.x() as f32 + 1.0,
            tile.y() as f32 + 1.0,
        ),
        DoorDirection::Vertical => Rect::new(
            tile.x() as f32,
            tile.y() as f32,
            tile.x() as f32 + 1.0,
            if walls.contains(Adjacency::NORTH) {
                tile.y() as f32 + 1.572
            } else {
                tile.y() as f32 + 2.0
            },
        ),
    }
}
