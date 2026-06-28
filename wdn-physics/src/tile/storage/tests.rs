use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::prelude::*;

use crate::{
    layer::Layer,
    tile::{
        CHUNK_SIZE_SQUARED, TilePlugin,
        material::TileMaterial,
        position::{TileChunkOffset, TileChunkPosition, TilePosition},
        storage::{Adjacency, TileChunk, TileKind, TileMap, TileStorage, TileStorageMut},
    },
};

#[test]
fn tile_chunk_empty() {
    let position = TileChunkPosition::new(Entity::PLACEHOLDER, 0, 0);
    let chunk = TileChunk::empty(position);

    assert_eq!(chunk.position(), position);
    assert_eq!(chunk.tiles().len(), CHUNK_SIZE_SQUARED);

    for (_, tile) in chunk.tiles() {
        assert_eq!(tile.material, TileMaterial::EMPTY);
    }
}

#[test]
fn tile_chunk_material_access() {
    let position = TileChunkPosition::new(Entity::PLACEHOLDER, 0, 0);
    let mut chunk = TileChunk::empty(position);
    let offset = TileChunkOffset::new(10, 1);

    assert_eq!(chunk.get(offset).material, TileMaterial::EMPTY);

    chunk.get_mut(offset).material = TileMaterial::WALL;
    assert_eq!(chunk.get(offset).material, TileMaterial::WALL);
}

#[test]
fn tile_storage_chunk_not_found() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            let tile_pos = TilePosition::new(layer, 0, 0);
            assert!(storage.get(tile_pos).is_none());
            assert_eq!(storage.get_kind(tile_pos), TileKind::Empty);
        })
        .unwrap();
}

#[test]
fn tile_storage_set_material() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            let tile_pos = TilePosition::new(layer, 5, 7);
            storage.set_material(tile_pos, TileMaterial::WALL);
            assert!(storage.get(tile_pos).is_some());
            assert_eq!(storage.get_kind(tile_pos), TileKind::Wall);
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            let tile_pos = TilePosition::new(layer, 5, 7);
            assert!(storage.get(tile_pos).is_some());
            assert_eq!(storage.get_kind(tile_pos), TileKind::Wall);
        })
        .unwrap();
}

#[test]
fn tile_storage_set_range() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let range: Vec<_> = (-100..100)
        .flat_map(|x| (-100..100).map(move |y| TilePosition::new(layer, x, y)))
        .collect();
    let range_clone = range.clone();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            for &tile in &range {
                storage.set_material(tile, TileMaterial::WALL);
            }

            for &tile in &range {
                assert!(storage.get(tile).is_some());
                assert_eq!(storage.get_kind(tile), TileKind::Wall);
            }
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            for &tile in &range_clone {
                assert!(storage.get(tile).is_some());
                assert_eq!(storage.get_kind(tile), TileKind::Wall);
            }
        })
        .unwrap();
}

#[test]
fn tile_storage_multiple_layers() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer1 = app.world_mut().spawn(Layer::default()).id();
    let layer2 = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            let position = IVec2::new(10, 15);
            let tile1 = TilePosition::from_vec(layer1, position);
            let tile2 = TilePosition::from_vec(layer2, position);

            storage.set_material(tile1, TileMaterial::WALL);
            storage.set_material(tile2, TileMaterial::EMPTY);

            assert_eq!(storage.get_kind(tile1), TileKind::Wall);
            assert_eq!(storage.get_kind(tile2), TileKind::Empty);
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            let position = IVec2::new(10, 15);
            let tile1 = TilePosition::from_vec(layer1, position);
            let tile2 = TilePosition::from_vec(layer2, position);

            assert_eq!(storage.get_kind(tile1), TileKind::Wall);
            assert_eq!(storage.get_kind(tile2), TileKind::Empty);
        })
        .unwrap();
}

#[test]
fn tile_storage_remove_chunk() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let position = TileChunkPosition::new(layer, 5, 5);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.chunk_mut(position);
        })
        .unwrap();

    let chunk_entity = app.world_mut().resource::<TileMap>().get(position).unwrap();

    assert!(app.world_mut().get_entity(chunk_entity).is_ok());

    app.world_mut().entity_mut(layer).despawn();

    assert!(app.world_mut().get_entity(chunk_entity).is_err());

    assert!(
        app.world_mut()
            .resource::<TileMap>()
            .get(position)
            .is_none()
    );

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            let tile_pos = TilePosition::new(layer, 5, 5);
            assert!(storage.get(tile_pos).is_none());
        })
        .unwrap();
}

#[test]
fn wall_adjacency() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let center = TilePosition::new(layer, 5, 5);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(center, TileMaterial::WALL);

            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                Adjacency::WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                Adjacency::EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                Adjacency::NORTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                Adjacency::NORTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                Adjacency::NORTH_WEST
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                Adjacency::WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                Adjacency::EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                Adjacency::NORTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                Adjacency::NORTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                Adjacency::NORTH_WEST
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(center, TileMaterial::EMPTY);

            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                Adjacency::NONE
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn door_adjacency() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let center = TilePosition::new(layer, 5, 5);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(center, TileMaterial::DOOR);

            assert_eq!(
                storage.get(center).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::WEST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 4, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::EAST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 4))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NORTH
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH_WEST
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            assert_eq!(
                storage.get(center).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::WEST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 4, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::EAST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 4))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NORTH
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH_WEST
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(center, TileMaterial::EMPTY);

            assert_eq!(
                storage.get(center).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 4, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 4))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            assert_eq!(
                storage.get(center).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 4, 5))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 5, 4))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 6, 6))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn door_adjacency_chunk_edge_horizontal() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let edge_tile = TilePosition::new(layer, 31, 15);
    let east_neighbor = TilePosition::new(layer, 32, 15);
    let west_neighbor = TilePosition::new(layer, 30, 15);

    assert_eq!(edge_tile.chunk_position(), west_neighbor.chunk_position());
    assert_ne!(edge_tile.chunk_position(), east_neighbor.chunk_position());

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge_tile, TileMaterial::DOOR);

            assert_eq!(
                storage.get(east_neighbor).unwrap().door_adjacency(),
                Adjacency::WEST
            );
            assert_eq!(
                storage.get(west_neighbor).unwrap().door_adjacency(),
                Adjacency::EAST
            );

            storage.set_material(edge_tile, TileMaterial::EMPTY);

            assert_eq!(
                storage.get(east_neighbor).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(west_neighbor).unwrap().door_adjacency(),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn door_adjacency_chunk_edge_vertical() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let edge_tile = TilePosition::new(layer, 15, 31);
    let north_neighbor = TilePosition::new(layer, 15, 32);
    let south_neighbor = TilePosition::new(layer, 15, 30);

    assert_eq!(edge_tile.chunk_position(), south_neighbor.chunk_position());
    assert_ne!(edge_tile.chunk_position(), north_neighbor.chunk_position());

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge_tile, TileMaterial::DOOR);

            assert_eq!(
                storage.get(north_neighbor).unwrap().door_adjacency(),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage.get(south_neighbor).unwrap().door_adjacency(),
                Adjacency::NORTH
            );

            storage.set_material(edge_tile, TileMaterial::EMPTY);

            assert_eq!(
                storage.get(north_neighbor).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(south_neighbor).unwrap().door_adjacency(),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn door_adjacency_chunk_corner() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let corner_tile = TilePosition::new(layer, 0, 0);
    let east_tile = TilePosition::new(layer, 1, 0);
    let north_tile = TilePosition::new(layer, 0, 1);
    let west_tile = TilePosition::new(layer, -1, 0);
    let south_tile = TilePosition::new(layer, 0, -1);

    assert_eq!(corner_tile.chunk_position(), east_tile.chunk_position());
    assert_eq!(corner_tile.chunk_position(), north_tile.chunk_position());
    assert_ne!(corner_tile.chunk_position(), west_tile.chunk_position());
    assert_ne!(corner_tile.chunk_position(), south_tile.chunk_position());

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(corner_tile, TileMaterial::DOOR);

            assert_eq!(
                storage.get(corner_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(east_tile).unwrap().door_adjacency(),
                Adjacency::WEST
            );
            assert_eq!(
                storage.get(north_tile).unwrap().door_adjacency(),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage.get(west_tile).unwrap().door_adjacency(),
                Adjacency::EAST
            );
            assert_eq!(
                storage.get(south_tile).unwrap().door_adjacency(),
                Adjacency::NORTH
            );

            storage.set_material(corner_tile, TileMaterial::EMPTY);

            assert_eq!(
                storage.get(corner_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(east_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(north_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(west_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get(south_tile).unwrap().door_adjacency(),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn door_adjacency_multiple_doors() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(TilePosition::new(layer, 10, 10), TileMaterial::DOOR);
            storage.set_material(TilePosition::new(layer, 11, 10), TileMaterial::DOOR);
            storage.set_material(TilePosition::new(layer, 10, 11), TileMaterial::DOOR);

            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 10, 10))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NORTH | Adjacency::EAST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 11, 10))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::NORTH_WEST | Adjacency::WEST
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 10, 11))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH_EAST | Adjacency::SOUTH
            );
            assert_eq!(
                storage
                    .get(TilePosition::new(layer, 11, 11))
                    .unwrap()
                    .door_adjacency(),
                Adjacency::SOUTH | Adjacency::SOUTH_WEST | Adjacency::WEST
            );
        })
        .unwrap();
}

#[test]
fn wall_adjacency_chunk_edge_horizontal() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let edge_tile = TilePosition::new(layer, 31, 15);
    let east_neighbor = TilePosition::new(layer, 32, 15);
    let west_neighbor = TilePosition::new(layer, 30, 15);
    let northeast_neighbor = TilePosition::new(layer, 32, 16);
    let southeast_neighbor = TilePosition::new(layer, 32, 14);

    assert_eq!(edge_tile.chunk_position(), west_neighbor.chunk_position());

    assert_ne!(edge_tile.chunk_position(), east_neighbor.chunk_position());
    assert_ne!(
        edge_tile.chunk_position(),
        northeast_neighbor.chunk_position()
    );
    assert_ne!(
        edge_tile.chunk_position(),
        southeast_neighbor.chunk_position()
    );

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge_tile, TileMaterial::WALL);

            assert_eq!(storage.get_wall_adjacency(east_neighbor), Adjacency::WEST);
            assert_eq!(storage.get_wall_adjacency(west_neighbor), Adjacency::EAST);
            assert_eq!(
                storage.get_wall_adjacency(northeast_neighbor),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(southeast_neighbor),
                Adjacency::NORTH_WEST
            );

            storage.set_material(edge_tile, TileMaterial::EMPTY);
            assert_eq!(storage.get_wall_adjacency(east_neighbor), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(west_neighbor), Adjacency::NONE);
            assert_eq!(
                storage.get_wall_adjacency(northeast_neighbor),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(southeast_neighbor),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn wall_adjacency_chunk_edge_vertical() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let edge_tile = TilePosition::new(layer, 15, 31);
    let north_neighbor = TilePosition::new(layer, 15, 32);
    let south_neighbor = TilePosition::new(layer, 15, 30);
    let northwest_neighbor = TilePosition::new(layer, 14, 32);
    let northeast_neighbor = TilePosition::new(layer, 16, 32);

    assert_eq!(edge_tile.chunk_position(), south_neighbor.chunk_position());

    assert_ne!(edge_tile.chunk_position(), north_neighbor.chunk_position());
    assert_ne!(
        edge_tile.chunk_position(),
        northwest_neighbor.chunk_position()
    );
    assert_ne!(
        edge_tile.chunk_position(),
        northeast_neighbor.chunk_position()
    );

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge_tile, TileMaterial::WALL);

            assert_eq!(storage.get_wall_adjacency(north_neighbor), Adjacency::SOUTH);
            assert_eq!(storage.get_wall_adjacency(south_neighbor), Adjacency::NORTH);
            assert_eq!(
                storage.get_wall_adjacency(northwest_neighbor),
                Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(northeast_neighbor),
                Adjacency::SOUTH_WEST
            );

            storage.set_material(edge_tile, TileMaterial::EMPTY);

            assert_eq!(storage.get_wall_adjacency(north_neighbor), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(south_neighbor), Adjacency::NONE);
            assert_eq!(
                storage.get_wall_adjacency(northwest_neighbor),
                Adjacency::NONE
            );
            assert_eq!(
                storage.get_wall_adjacency(northeast_neighbor),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn wall_adjacency_chunk_corner() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let corner_tile = TilePosition::new(layer, 0, 0);
    let east_tile = TilePosition::new(layer, 1, 0);
    let northeast_tile = TilePosition::new(layer, 1, 1);
    let north_tile = TilePosition::new(layer, 0, 1);
    let northwest_tile = TilePosition::new(layer, -1, 1);
    let west_tile = TilePosition::new(layer, -1, 0);
    let southwest_tile = TilePosition::new(layer, -1, -1);
    let south_tile = TilePosition::new(layer, 0, -1);
    let southeast_tile = TilePosition::new(layer, 1, -1);

    assert_eq!(corner_tile.chunk_position(), east_tile.chunk_position());
    assert_eq!(corner_tile.chunk_position(), north_tile.chunk_position());
    assert_eq!(
        corner_tile.chunk_position(),
        northeast_tile.chunk_position()
    );

    assert_ne!(corner_tile.chunk_position(), west_tile.chunk_position());
    assert_ne!(corner_tile.chunk_position(), south_tile.chunk_position());
    assert_ne!(
        corner_tile.chunk_position(),
        northwest_tile.chunk_position()
    );
    assert_ne!(
        corner_tile.chunk_position(),
        southwest_tile.chunk_position()
    );
    assert_ne!(
        corner_tile.chunk_position(),
        southeast_tile.chunk_position()
    );

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(corner_tile, TileMaterial::WALL);

            assert_eq!(storage.get_wall_adjacency(corner_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(east_tile), Adjacency::WEST);
            assert_eq!(
                storage.get_wall_adjacency(northeast_tile),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(storage.get_wall_adjacency(north_tile), Adjacency::SOUTH);
            assert_eq!(
                storage.get_wall_adjacency(northwest_tile),
                Adjacency::SOUTH_EAST
            );
            assert_eq!(storage.get_wall_adjacency(west_tile), Adjacency::EAST);
            assert_eq!(
                storage.get_wall_adjacency(southwest_tile),
                Adjacency::NORTH_EAST
            );
            assert_eq!(storage.get_wall_adjacency(south_tile), Adjacency::NORTH);
            assert_eq!(
                storage.get_wall_adjacency(southeast_tile),
                Adjacency::NORTH_WEST
            );
            assert_eq!(storage.get_wall_adjacency(south_tile), Adjacency::NORTH);
            assert_eq!(
                storage.get_wall_adjacency(southeast_tile),
                Adjacency::NORTH_WEST
            );

            storage.set_material(corner_tile, TileMaterial::EMPTY);

            assert_eq!(storage.get_wall_adjacency(corner_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(east_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(northeast_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(north_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(northwest_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(west_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(southwest_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(south_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(southeast_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(south_tile), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(southeast_tile), Adjacency::NONE);
        })
        .unwrap();
}

#[test]
fn wall_adjacency_multiple_solid_tiles() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(TilePosition::new(layer, 10, 10), TileMaterial::WALL);
            storage.set_material(TilePosition::new(layer, 11, 10), TileMaterial::WALL);
            storage.set_material(TilePosition::new(layer, 10, 11), TileMaterial::WALL);

            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 9, 9)),
                Adjacency::NORTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 10, 9)),
                Adjacency::NORTH | Adjacency::NORTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 11, 9)),
                Adjacency::NORTH_WEST | Adjacency::NORTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 12, 9)),
                Adjacency::NORTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 9, 10)),
                Adjacency::NORTH_EAST | Adjacency::EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 10, 10)),
                Adjacency::NORTH | Adjacency::EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 11, 10)),
                Adjacency::NORTH_WEST | Adjacency::WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 12, 10)),
                Adjacency::WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 9, 11)),
                Adjacency::EAST | Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 10, 11)),
                Adjacency::SOUTH | Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 11, 11)),
                Adjacency::WEST | Adjacency::SOUTH_WEST | Adjacency::SOUTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 12, 11)),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 9, 12)),
                Adjacency::SOUTH_EAST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 10, 12)),
                Adjacency::SOUTH
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 11, 12)),
                Adjacency::SOUTH_WEST
            );
            assert_eq!(
                storage.get_wall_adjacency(TilePosition::new(layer, 12, 12)),
                Adjacency::NONE
            );
        })
        .unwrap();
}

#[test]
fn wall_adjacency_overwrite() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            let center = TilePosition::new(layer, 5, 5);
            let east_neighbor = center.east();

            storage.set_material(center, TileMaterial::WALL);
            storage.set_material(east_neighbor, TileMaterial::EMPTY);

            assert_eq!(storage.get_wall_adjacency(center), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(east_neighbor), Adjacency::WEST);

            storage.set_material(center, TileMaterial::WALL);
            storage.set_material(east_neighbor, TileMaterial::EMPTY);

            assert_eq!(storage.get_wall_adjacency(center), Adjacency::NONE);
            assert_eq!(storage.get_wall_adjacency(east_neighbor), Adjacency::WEST);
        })
        .unwrap();
}

#[test]
fn tile_storage_change_detection() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let tile1 = TilePosition::new(layer, 5, 5);
    let tile2 = TilePosition::new(layer, -5, 5);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(tile1, TileMaterial::WALL);
            storage.set_material(tile2, TileMaterial::WALL);
        })
        .unwrap();

    let chunk1 = app
        .world()
        .resource::<TileMap>()
        .get(tile1.chunk_position())
        .unwrap();
    let chunk2 = app
        .world()
        .resource::<TileMap>()
        .get(tile2.chunk_position())
        .unwrap();

    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_added()
    );
    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_added()
    );

    app.world_mut().clear_trackers();

    assert!(
        !app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_changed()
    );
    assert!(
        !app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_changed()
    );

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(tile1, TileMaterial::EMPTY);
        })
        .unwrap();

    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_changed()
    );
    assert!(
        !app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_changed()
    );
}

#[test]
fn tile_storage_change_detection_border() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let edge = TilePosition::new(layer, 0, 5);

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge, TileMaterial::WALL);
        })
        .unwrap();

    let chunk1 = app
        .world()
        .resource::<TileMap>()
        .get(edge.chunk_position())
        .unwrap();
    let chunk2 = app
        .world()
        .resource::<TileMap>()
        .get(edge.west().chunk_position())
        .unwrap();
    assert_ne!(chunk1, chunk2);

    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_added()
    );
    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_added()
    );

    app.world_mut().clear_trackers();

    assert!(
        !app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_changed()
    );
    assert!(
        !app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_changed()
    );

    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(edge, TileMaterial::EMPTY);
        })
        .unwrap();

    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk1)
            .unwrap()
            .is_changed()
    );
    assert!(
        app.world_mut()
            .get_mut::<TileChunk>(chunk2)
            .unwrap()
            .is_changed()
    );
}

#[test]
fn tile_storage_nested_buffer() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let tile1 = TilePosition::new(layer, 5, 5);
    let tile2 = TilePosition::new(layer, 10, 10);

    app.world_mut()
        .run_system_once(move |mut commands: Commands, mut storage: TileStorageMut| {
            storage.set_material(tile1, TileMaterial::WALL);

            commands.spawn((TilePosition::new(layer, 10, 10), TileMaterial::WALL));
        })
        .unwrap();

    app.world_mut()
        .run_system_once(move |storage: TileStorage| {
            assert_eq!(storage.get(tile1).unwrap().kind(), TileKind::Wall);
            assert_eq!(storage.get(tile2).unwrap().kind(), TileKind::Wall);
        })
        .unwrap();
}
