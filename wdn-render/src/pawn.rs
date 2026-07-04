use bevy_app::prelude::*;
use bevy_camera::visibility::Visibility;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_sprite::{Anchor, prelude::*};
use bevy_transform::prelude::*;
use wdn_world::pawn::{Pawn, PawnProjectile};

use crate::{assets::AssetHandles, depth::PAWN_DEPTH, lerp::Interpolate};

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Visibility, Transform)]
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
        world.commands().spawn((
            ChildOf(context.entity),
            sprite,
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, -Pawn::RADIUS, PAWN_DEPTH),
        ));
    }
}

impl PawnProjectileSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let sprite = world.resource::<AssetHandles>().pawn_projectile();
        *world.get_mut::<Sprite>(context.entity).unwrap() = sprite;

        world
            .get_mut::<Transform>(context.entity)
            .unwrap()
            .translation
            .z = PAWN_DEPTH;
    }
}
