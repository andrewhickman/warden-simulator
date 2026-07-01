use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use bevy_math::prelude::*;

use crate::{
    layer::Layer,
    tile::{
        TilePlugin,
        adjacency::{Adjacency, TileAdjacency},
        index::TileIndex,
        material::TileMaterial,
        position::TilePosition,
        storage::TileStorageMut,
    },
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
    let entities = index.get_objects(TilePosition::new(layer, 1, -1));
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
    assert!(
        index
            .get_objects(TilePosition::new(layer, 2, -1))
            .is_empty()
    );

    app.world_mut()
        .entity_mut(entity)
        .insert(TilePosition::new(layer, 2, -1));

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get_objects(TilePosition::new(layer, 2, -1));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get_objects(TilePosition::new(layer, 1, -1));
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
    assert_eq!(index.get_objects(TilePosition::new(layer, 1, -1)).len(), 1);

    app.world_mut().entity_mut(entity).despawn();

    let index = app.world().resource::<TileIndex>();
    let entities = index.get_objects(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[]);
}

#[test]
fn tile_position_sync_material_adjacency() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let center = TilePosition::new(layer, 4, 4);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(center, TileMaterial::WALL);
            storage.set_material(center.east(), TileMaterial::WALL);
            storage.set_material(center.west(), TileMaterial::DOOR);
        })
        .unwrap();

    let entity = app
        .world_mut()
        .spawn((
            ChildOf(layer),
            TilePosition::new(layer, 4, 4),
            TileMaterial::EMPTY,
            TileAdjacency::NONE,
        ))
        .id();

    let material = app.world().get::<TileMaterial>(entity).copied().unwrap();
    let adjacency = app.world().get::<TileAdjacency>(entity).copied().unwrap();

    assert_eq!(material, TileMaterial::WALL);
    assert_eq!(adjacency.walls(), Adjacency::EAST);
    assert_eq!(adjacency.doors(), Adjacency::WEST);

    app.world_mut().entity_mut(entity).insert(center.south());

    let material = app.world().get::<TileMaterial>(entity).copied().unwrap();
    let adjacency = app.world().get::<TileAdjacency>(entity).copied().unwrap();

    assert_eq!(material, TileMaterial::EMPTY);
    assert_eq!(adjacency.walls(), Adjacency::NORTH | Adjacency::NORTH_EAST);
    assert_eq!(adjacency.doors(), Adjacency::NORTH_WEST);
}
