use bevy_app::prelude::*;
use bevy_camera::visibility::Visibility;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_sprite::{Anchor, prelude::*};

use bevy_transform::components::Transform;
use wdn_physics::tile::TilePosition;
use wdn_world::{
    door::Door,
    pawn::{Pawn, PawnProjectile},
};

use crate::{
    assets::AssetHandles,
    layers::{DOOR_LAYER, PAWN_LAYER},
    lerp::Interpolate,
};

pub struct PawnPlugin;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Visibility, Transform)]
#[component(on_add = PawnSprite::on_add)]
pub struct PawnSprite;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Sprite)]
#[component(on_add = PawnProjectileSprite::on_add)]
pub struct PawnProjectileSprite;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Visibility, Transform)]
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
        world.commands().spawn((
            ChildOf(context.entity),
            sprite,
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, -Pawn::RADIUS, PAWN_LAYER),
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
            .z = PAWN_LAYER;
    }
}

impl DoorSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let handles = world.resource::<AssetHandles>();
        let door = handles.door();
        let left = handles.door_left();
        let right = handles.door_right();

        let tile = *world.get::<TilePosition>(context.entity).unwrap();
        *world.get_mut::<Transform>(context.entity).unwrap() =
            Transform::from_xyz(tile.x() as f32 + 0.5, tile.y() as f32, DOOR_LAYER);

        world.commands().spawn_batch([
            (
                ChildOf(context.entity),
                door,
                Anchor::CENTER,
                Transform::from_xyz(0.0, 0.65625, 0.0),
            ),
            (
                ChildOf(context.entity),
                left,
                Anchor::BOTTOM_RIGHT,
                Transform::from_xyz(-0.5, 0.1875, 0.0),
            ),
            (
                ChildOf(context.entity),
                right,
                Anchor::BOTTOM_LEFT,
                Transform::from_xyz(0.5, 0.1875, 0.0),
            ),
        ]);
    }
}
