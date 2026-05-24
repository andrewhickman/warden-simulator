use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_math::prelude::*;
use bevy_sprite::{Anchor, prelude::*};
use bevy_transform::prelude::*;
use wdn_physics::tile::TilePosition;
use wdn_world::door::Door;

use crate::{assets::AssetHandles, layers::SPRITE_LAYER};

pub struct DoorPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite, Anchor::BOTTOM_LEFT)]
#[component(on_add = DoorSprite::on_add)]
pub struct DoorSprite;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Door, DoorSprite>();
    }
}

impl DoorSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let tile = *world.get::<TilePosition>(context.entity).unwrap();

        let sprite = world.resource::<AssetHandles>().door_horizontal();
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;

        world
            .get_mut::<Transform>(context.entity)
            .unwrap()
            .translation = Vec3::new(tile.x() as f32, tile.y() as f32 + 0.12, SPRITE_LAYER);
    }
}
