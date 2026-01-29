use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_sprite::prelude::*;

use bevy_transform::components::Transform;
use wdn_physics::tile::TilePosition;
use wdn_world::{
    door::Door,
    pawn::{Pawn, PawnProjectile},
};

use crate::{assets::AssetHandles, lerp::Interpolate};

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = PawnSprite::on_add)]
pub struct PawnSprite;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = PawnProjectileSprite::on_add)]
pub struct PawnProjectileSprite;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = DoorSprite::on_add)]
pub struct DoorSprite;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Pawn, PawnSprite>();
        app.register_required_components_with::<Pawn, Interpolate>(Interpolate::translation);
        app.register_required_components::<PawnProjectile, PawnProjectileSprite>();
        app.register_required_components_with::<PawnProjectile, Interpolate>(
            Interpolate::translation,
        );
        app.register_required_components::<Door, DoorSprite>();
    }
}

impl PawnSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let sprite = world.resource::<AssetHandles>().pawn();
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;
    }
}

impl PawnProjectileSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let sprite = world.resource::<AssetHandles>().pawn_projectile();
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;
    }
}

impl DoorSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let sprite = world.resource::<AssetHandles>().door();
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;

        let tile = *world.get::<TilePosition>(context.entity).unwrap();
        *world.get_mut::<Transform>(context.entity).unwrap() = tile_transform(tile);
    }
}

fn tile_transform(position: TilePosition) -> Transform {
    Transform::from_xyz(position.x() as f32 + 0.5, position.y() as f32 + 0.5, 0.0)
}
