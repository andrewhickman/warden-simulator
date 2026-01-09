use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;

use crate::{
    layer::Layer,
    tile::{TilePlugin, TilePosition, index::TileIndex},
};

#[test]
fn tile_position_added() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((ChildOf(layer), TilePosition::new(layer, 1, -1)))
        .id();

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
}

#[test]
fn tile_position_replaced() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((ChildOf(layer), TilePosition::new(layer, 1, -1)))
        .id();

    let index = app.world().resource::<TileIndex>();
    assert!(index.get(TilePosition::new(layer, 2, -1)).is_empty());

    app.world_mut()
        .entity_mut(entity)
        .insert(TilePosition::new(layer, 2, -1));

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 2, -1));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(prev_entities, &[]);
}

#[test]
fn tile_position_removed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((ChildOf(layer), TilePosition::new(layer, 1, -1)))
        .id();

    let index = app.world().resource::<TileIndex>();
    assert_eq!(index.get(TilePosition::new(layer, 1, -1)).len(), 1);

    app.world_mut().entity_mut(entity).despawn();

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[]);
}
