use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_math::prelude::*;
use bevy_sprite::prelude::*;

use wdn_physics::pawn::Pawn;

use crate::assets::AssetHandles;

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = PawnSprite::on_add)]
pub struct PawnSprite;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Pawn, PawnSprite>();
    }
}

impl PawnSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let image = world.resource::<AssetHandles>().pawn.clone();
        let mut sprite = world.get_mut::<Sprite>(context.entity).unwrap();
        sprite.image = image;
        sprite.custom_size = Some(Vec2::splat(Pawn::RADIUS as f32 * 3.6));
    }
}
