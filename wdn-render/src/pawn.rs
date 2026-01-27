use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_sprite::prelude::*;

use wdn_world::pawn::{Pawn, PawnProjectile};

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

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Pawn, PawnSprite>();
        app.register_required_components_with::<Pawn, Interpolate>(Interpolate::translation);
        app.register_required_components::<PawnProjectile, PawnProjectileSprite>();
        app.register_required_components_with::<PawnProjectile, Interpolate>(
            Interpolate::translation,
        );
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
