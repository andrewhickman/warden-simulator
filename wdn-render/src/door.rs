use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_sprite::{Anchor, prelude::*};
use bevy_sprite_render::SpriteSystems;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use wdn_physics::{
    kinematics::Position,
    tile::{TilePosition, storage::TileChunk},
};
use wdn_world::door::{Door, DoorDirection};

use crate::{
    RenderSystems,
    assets::{AssetHandles, DOOR_HORIZONTAL_RECT, sprite_size},
    layers::DOOR_LAYER,
    lerp::{FixedUpdateCount, InterpolateState},
};

pub struct DoorPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = DoorSprite::on_add)]
pub struct DoorSprite {
    position: InterpolateState<f32>,
}

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            RenderSystems::RenderDoors
                .before(TransformSystems::Propagate)
                .before(SpriteSystems::ComputeSlices),
        );

        app.register_required_components::<Door, DoorSprite>();

        app.add_systems(PostUpdate, update_doors.in_set(RenderSystems::RenderDoors));
    }
}

impl DoorSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let tile = *world.get::<TilePosition>(context.entity).unwrap();

        let direction = *world.get::<DoorDirection>(context.entity).unwrap();
        let (sprite, anchor, transform) = match direction {
            DoorDirection::Horizontal => (
                world.resource::<AssetHandles>().door_horizontal(),
                Anchor::BOTTOM_LEFT,
                Transform::from_xyz(tile.x() as f32, tile.y() as f32 + 0.12, DOOR_LAYER),
            ),
            DoorDirection::Vertical => (
                world.resource::<AssetHandles>().door_vertical(),
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(tile.x() as f32 + 0.5, tile.y() as f32 - 0.4, DOOR_LAYER),
            ),
        };
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;
        *world.get_mut::<Anchor>(context.entity).unwrap() = anchor;
        *world.get_mut::<Transform>(context.entity).unwrap() = transform;
    }
}

pub fn update_doors(
    updates: Res<FixedUpdateCount>,
    mut doors: Query<
        (
            &Door,
            &DoorDirection,
            &TilePosition,
            &mut DoorSprite,
            &mut Transform,
            &mut Sprite,
        ),
        (Without<Position>, Without<TileChunk>),
    >,
    time: Res<Time<Fixed>>,
) {
    let overstep = time.overstep_fraction();

    doors.par_iter_mut().for_each(
        |(door, direction, tile, mut state, mut transform, mut sprite)| {
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
