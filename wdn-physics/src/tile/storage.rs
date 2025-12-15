use std::{fmt, mem};

use bevy::{
    ecs::{
        lifecycle::HookContext,
        system::{SystemBuffer, SystemMeta, SystemParam},
        world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};
use bitflags::bitflags;

use crate::tile::{CHUNK_SIZE, TileChunkOffset, TileChunkPosition, TilePosition};

#[derive(SystemParam)]
pub struct TileStorage<'w, 's> {
    map: Res<'w, TileMap>,
    chunks: Query<'w, 's, &'static TileChunk>,
}

#[derive(SystemParam)]
pub struct TileStorageMut<'w, 's> {
    map: ResMut<'w, TileMap>,
    chunks: Query<'w, 's, &'static mut TileChunk>,
    buffer: Deferred<'s, TileStorageBuffer>,
}

#[derive(Component)]
pub struct TileLayer {}

#[derive(Component)]
#[component(on_add = TileChunk::on_add, on_remove = TileChunk::on_remove)]
pub struct TileChunk {
    position: TileChunkPosition,
    tiles: Box<[Tile; CHUNK_SIZE * CHUNK_SIZE]>,
}

#[derive(Default, Resource)]
pub struct TileMap {
    chunks: HashMap<(Entity, TileChunkPosition), Entity>,
}

#[derive(Default)]
pub struct TileStorageBuffer {
    chunks: HashMap<(Entity, TileChunkPosition), TileChunk>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Tile {
    material: TileMaterial,
    adjacency: TileAdjacency,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileAdjacency : u8 {
        const NONE = 0b0000_0000;
        const EAST = 0b0000_0001;
        const NORTH_EAST = 0b0000_0010;
        const NORTH = 0b0000_0100;
        const NORTH_WEST = 0b0000_1000;
        const WEST = 0b0001_0000;
        const SOUTH_WEST = 0b0010_0000;
        const SOUTH = 0b0100_0000;
        const SOUTH_EAST = 0b1000_0000;
    }
}

impl TileStorage<'_, '_> {
    pub fn get(&self, tile: TilePosition) -> Option<&Tile> {
        self.chunk(tile.layer(), tile.chunk_position())
            .map(|chunk| chunk.get(tile.chunk_offset()))
    }

    pub fn get_material(&self, tile: TilePosition) -> TileMaterial {
        match self.get(tile) {
            Some(t) => t.material,
            None => TileMaterial::Empty,
        }
    }

    pub fn get_adjacency(&self, tile: TilePosition) -> TileAdjacency {
        match self.get(tile) {
            Some(t) => t.adjacency,
            None => TileAdjacency::NONE,
        }
    }

    pub fn chunk(&'_ self, layer: Entity, position: TileChunkPosition) -> Option<&TileChunk> {
        if let Some(chunk_entity) = self.map.chunks.get(&(layer, position)) {
            Some(
                self.chunks
                    .get(*chunk_entity)
                    .expect("invalid chunk entity"),
            )
        } else {
            None
        }
    }
}

impl TileStorageMut<'_, '_> {
    pub fn get(&self, tile: TilePosition) -> Option<&Tile> {
        self.chunk(tile.layer(), tile.chunk_position())
            .map(|chunk| chunk.get(tile.chunk_offset()))
    }

    pub fn get_material(&self, tile: TilePosition) -> TileMaterial {
        match self.get(tile) {
            Some(t) => t.material,
            None => TileMaterial::Empty,
        }
    }

    pub fn get_adjacency(&self, tile: TilePosition) -> TileAdjacency {
        match self.get(tile) {
            Some(t) => t.adjacency,
            None => TileAdjacency::NONE,
        }
    }

    pub fn set_material(&'_ mut self, position: TilePosition, material: TileMaterial) {
        let tile = self
            .chunk_mut(position.layer(), position.chunk_position())
            .get_mut(position.chunk_offset());
        let prev_material = mem::replace(&mut tile.material, material);

        match (prev_material.is_solid(), material.is_solid()) {
            (false, true) => self.add_adjacent(position),
            (true, false) => self.remove_adjacent(position),
            _ => {}
        }
    }

    pub fn chunk(&'_ self, layer: Entity, position: TileChunkPosition) -> Option<&TileChunk> {
        if let Some(chunk_entity) = self.map.chunks.get(&(layer, position)) {
            Some(
                self.chunks
                    .get(*chunk_entity)
                    .expect("invalid chunk entity"),
            )
        } else {
            self.buffer.chunks.get(&(layer, position))
        }
    }

    fn chunk_mut(&'_ mut self, layer: Entity, position: TileChunkPosition) -> &mut TileChunk {
        if let Some(chunk_entity) = self.map.chunks.get(&(layer, position)) {
            self.chunks
                .get_mut(*chunk_entity)
                .expect("invalid chunk entity")
                .into_inner()
        } else {
            self.buffer
                .chunks
                .entry((layer, position))
                .or_insert_with(|| TileChunk::empty(position))
        }
    }

    fn add_adjacent(&mut self, position: TilePosition) {
        for (adj, offset) in TileAdjacency::OFFSETS {
            let neighbor_pos = TilePosition::new(position.layer(), position.position() - offset);

            let neighbour_tile = self
                .chunk_mut(position.layer(), neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.insert(adj);
        }
    }

    fn remove_adjacent(&mut self, position: TilePosition) {
        for (adj, offset) in TileAdjacency::OFFSETS {
            let neighbor_pos = TilePosition::new(position.layer(), position.position() - offset);

            let neighbour_tile = self
                .chunk_mut(position.layer(), neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.remove(adj);
        }
    }
}

impl TileChunk {
    pub fn empty(position: TileChunkPosition) -> Self {
        Self {
            position,
            tiles: Box::new([Tile::empty(); CHUNK_SIZE * CHUNK_SIZE]),
        }
    }

    pub fn position(&self) -> TileChunkPosition {
        self.position
    }

    pub fn get(&self, offset: TileChunkOffset) -> &Tile {
        &self.tiles[offset.index()]
    }

    pub fn get_mut(&mut self, offset: TileChunkOffset) -> &mut Tile {
        &mut self.tiles[offset.index()]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = &Tile> {
        self.tiles.iter()
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let chunk = world.entity(context.entity);
        let layer = chunk
            .get::<ChildOf>()
            .expect("missing ChildOf component for TileChunk")
            .parent();
        let chunk = chunk.get::<TileChunk>().unwrap().position();
        world
            .resource_mut::<TileMap>()
            .chunks
            .insert((layer, chunk), context.entity);
    }

    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let chunk = world.entity(context.entity);
        let layer = chunk
            .get::<ChildOf>()
            .expect("missing ChildOf component for TileChunk")
            .parent();
        let chunk = chunk.get::<TileChunk>().unwrap().position();
        world
            .resource_mut::<TileMap>()
            .chunks
            .remove(&(layer, chunk));
    }
}

impl fmt::Debug for TileChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileChunk")
            .field("x", &self.position.x())
            .field("y", &self.position.y())
            .finish_non_exhaustive()
    }
}

impl SystemBuffer for TileStorageBuffer {
    fn apply(&mut self, _: &SystemMeta, world: &mut World) {
        if !self.chunks.is_empty() {
            world.spawn_batch(
                self.chunks
                    .drain()
                    .map(|((layer, _), chunk)| (ChildOf(layer), chunk)),
            );
        }
    }
}

impl Tile {
    pub fn empty() -> Self {
        Self {
            material: TileMaterial::Empty,
            adjacency: TileAdjacency::NONE,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.material.is_solid()
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }
}

impl TileMaterial {
    pub fn is_solid(&self) -> bool {
        match self {
            TileMaterial::Empty => false,
            TileMaterial::Wall => true,
        }
    }
}

impl TileAdjacency {
    const OFFSETS: [(TileAdjacency, IVec2); 8] = [
        (TileAdjacency::EAST, IVec2::new(1, 0)),
        (TileAdjacency::NORTH_EAST, IVec2::new(1, 1)),
        (TileAdjacency::NORTH, IVec2::new(0, 1)),
        (TileAdjacency::NORTH_WEST, IVec2::new(-1, 1)),
        (TileAdjacency::WEST, IVec2::new(-1, 0)),
        (TileAdjacency::SOUTH_WEST, IVec2::new(-1, -1)),
        (TileAdjacency::SOUTH, IVec2::new(0, -1)),
        (TileAdjacency::SOUTH_EAST, IVec2::new(1, -1)),
    ];
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::RunSystemOnce,
        math::{I16Vec2, IVec2},
        prelude::*,
    };

    use crate::tile::{
        CHUNK_SIZE, TileChunkOffset, TileChunkPosition, TilePlugin, TilePosition,
        storage::{
            TileAdjacency, TileChunk, TileLayer, TileMap, TileMaterial, TileStorage, TileStorageMut,
        },
    };

    #[test]
    fn tile_chunk_empty() {
        let position = TileChunkPosition(I16Vec2::ZERO);
        let chunk = TileChunk::empty(position);

        assert_eq!(chunk.position(), position);
        assert_eq!(chunk.tiles().len(), CHUNK_SIZE * CHUNK_SIZE);

        for tile in chunk.tiles() {
            assert_eq!(tile.material, TileMaterial::Empty);
        }
    }

    #[test]
    fn tile_chunk_material_access() {
        let position = TileChunkPosition(I16Vec2::ZERO);
        let mut chunk = TileChunk::empty(position);
        let offset = TileChunkOffset::new(10, 1);

        assert_eq!(chunk.get(offset).material, TileMaterial::Empty);

        chunk.get_mut(offset).material = TileMaterial::Wall;
        assert_eq!(chunk.get(offset).material, TileMaterial::Wall);
    }

    #[test]
    fn tile_storage_chunk_not_found() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, IVec2::new(0, 0));
                assert!(storage.get(tile_pos).is_none());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Empty);
            })
            .unwrap();
    }

    #[test]
    fn tile_storage_set_material() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                let tile_pos = TilePosition::new(layer, IVec2::new(5, 7));
                storage.set_material(tile_pos, TileMaterial::Wall);
                assert!(storage.get(tile_pos).is_some());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Wall);
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, IVec2::new(5, 7));
                assert!(storage.get(tile_pos).is_some());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Wall);
            })
            .unwrap();
    }

    #[test]
    fn tile_storage_set_range() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        let range: Vec<_> = (-100..100)
            .flat_map(|x| (-100..100).map(move |y| TilePosition::new(layer, IVec2::new(x, y))))
            .collect();
        let range_clone = range.clone();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                for &tile in &range {
                    storage.set_material(tile, TileMaterial::Wall);
                }

                for &tile in &range {
                    assert!(storage.get(tile).is_some());
                    assert_eq!(storage.get_material(tile), TileMaterial::Wall);
                }
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                for &tile in &range_clone {
                    assert!(storage.get(tile).is_some());
                    assert_eq!(storage.get_material(tile), TileMaterial::Wall);
                }
            })
            .unwrap();
    }

    #[test]
    fn tile_storage_multiple_layers() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer1 = app.world_mut().spawn(TileLayer {}).id();
        let layer2 = app.world_mut().spawn(TileLayer {}).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                let position = IVec2::new(10, 15);
                let tile1 = TilePosition::new(layer1, position);
                let tile2 = TilePosition::new(layer2, position);

                storage.set_material(tile1, TileMaterial::Wall);
                storage.set_material(tile2, TileMaterial::Empty);

                assert_eq!(storage.get_material(tile1), TileMaterial::Wall);
                assert_eq!(storage.get_material(tile2), TileMaterial::Empty);
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let position = IVec2::new(10, 15);
                let tile1 = TilePosition::new(layer1, position);
                let tile2 = TilePosition::new(layer2, position);

                assert_eq!(storage.get_material(tile1), TileMaterial::Wall);
                assert_eq!(storage.get_material(tile2), TileMaterial::Empty);
            })
            .unwrap();
    }

    #[test]
    fn tile_storage_remove_chunk() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();
        let position = TileChunkPosition::new(5, 5);

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.chunk_mut(layer, position);
            })
            .unwrap();

        let chunk_entity = *app
            .world_mut()
            .resource::<TileMap>()
            .chunks
            .get(&(layer, position))
            .unwrap();

        assert!(app.world_mut().get_entity(chunk_entity).is_ok());

        app.world_mut().entity_mut(layer).despawn();

        assert!(app.world_mut().get_entity(chunk_entity).is_err());

        assert!(
            !app.world_mut()
                .resource::<TileMap>()
                .chunks
                .contains_key(&(layer, position))
        );

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, IVec2::new(5, 5));
                assert!(storage.get(tile_pos).is_none());
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        let center = TilePosition::new(layer, IVec2::new(5, 5));

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(center, TileMaterial::Wall);

                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 5))),
                    TileAdjacency::WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 6))),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 6))),
                    TileAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 6))),
                    TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 5))),
                    TileAdjacency::EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 4))),
                    TileAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 4))),
                    TileAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 4))),
                    TileAdjacency::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 5))),
                    TileAdjacency::WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 6))),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 6))),
                    TileAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 6))),
                    TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 5))),
                    TileAdjacency::EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 4))),
                    TileAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 4))),
                    TileAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 4))),
                    TileAdjacency::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(center, TileMaterial::Empty);

                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 4))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 4))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 4))),
                    TileAdjacency::NONE
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 6))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 5))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(4, 4))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(5, 4))),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(6, 4))),
                    TileAdjacency::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency_chunk_edge_horizontal() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        let edge_tile = TilePosition::new(layer, IVec2::new(31, 15));
        let east_neighbor = TilePosition::new(layer, IVec2::new(32, 15));
        let west_neighbor = TilePosition::new(layer, IVec2::new(30, 15));
        let northeast_neighbor = TilePosition::new(layer, IVec2::new(32, 16));
        let southeast_neighbor = TilePosition::new(layer, IVec2::new(32, 14));

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
                storage.set_material(edge_tile, TileMaterial::Wall);

                assert_eq!(storage.get_adjacency(east_neighbor), TileAdjacency::WEST);
                assert_eq!(storage.get_adjacency(west_neighbor), TileAdjacency::EAST);
                assert_eq!(
                    storage.get_adjacency(northeast_neighbor),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(southeast_neighbor),
                    TileAdjacency::NORTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);
                assert_eq!(storage.get_adjacency(east_neighbor), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(west_neighbor), TileAdjacency::NONE);
                assert_eq!(
                    storage.get_adjacency(northeast_neighbor),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(southeast_neighbor),
                    TileAdjacency::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency_chunk_edge_vertical() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        let edge_tile = TilePosition::new(layer, IVec2::new(15, 31));
        let north_neighbor = TilePosition::new(layer, IVec2::new(15, 32));
        let south_neighbor = TilePosition::new(layer, IVec2::new(15, 30));
        let northwest_neighbor = TilePosition::new(layer, IVec2::new(14, 32));
        let northeast_neighbor = TilePosition::new(layer, IVec2::new(16, 32));

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
                storage.set_material(edge_tile, TileMaterial::Wall);

                assert_eq!(storage.get_adjacency(north_neighbor), TileAdjacency::SOUTH);
                assert_eq!(storage.get_adjacency(south_neighbor), TileAdjacency::NORTH);
                assert_eq!(
                    storage.get_adjacency(northwest_neighbor),
                    TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(northeast_neighbor),
                    TileAdjacency::SOUTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);

                assert_eq!(storage.get_adjacency(north_neighbor), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(south_neighbor), TileAdjacency::NONE);
                assert_eq!(
                    storage.get_adjacency(northwest_neighbor),
                    TileAdjacency::NONE
                );
                assert_eq!(
                    storage.get_adjacency(northeast_neighbor),
                    TileAdjacency::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency_chunk_corner() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        let corner_tile = TilePosition::new(layer, IVec2::new(0, 0));
        let east_tile = TilePosition::new(layer, IVec2::new(1, 0));
        let northeast_tile = TilePosition::new(layer, IVec2::new(1, 1));
        let north_tile = TilePosition::new(layer, IVec2::new(0, 1));
        let northwest_tile = TilePosition::new(layer, IVec2::new(-1, 1));
        let west_tile = TilePosition::new(layer, IVec2::new(-1, 0));
        let southwest_tile = TilePosition::new(layer, IVec2::new(-1, -1));
        let south_tile = TilePosition::new(layer, IVec2::new(0, -1));
        let southeast_tile = TilePosition::new(layer, IVec2::new(1, -1));

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
                storage.set_material(corner_tile, TileMaterial::Wall);

                assert_eq!(storage.get_adjacency(corner_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(east_tile), TileAdjacency::WEST);
                assert_eq!(
                    storage.get_adjacency(northeast_tile),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(storage.get_adjacency(north_tile), TileAdjacency::SOUTH);
                assert_eq!(
                    storage.get_adjacency(northwest_tile),
                    TileAdjacency::SOUTH_EAST
                );
                assert_eq!(storage.get_adjacency(west_tile), TileAdjacency::EAST);
                assert_eq!(
                    storage.get_adjacency(southwest_tile),
                    TileAdjacency::NORTH_EAST
                );
                assert_eq!(storage.get_adjacency(south_tile), TileAdjacency::NORTH);
                assert_eq!(
                    storage.get_adjacency(southeast_tile),
                    TileAdjacency::NORTH_WEST
                );
                assert_eq!(storage.get_adjacency(south_tile), TileAdjacency::NORTH);
                assert_eq!(
                    storage.get_adjacency(southeast_tile),
                    TileAdjacency::NORTH_WEST
                );

                storage.set_material(corner_tile, TileMaterial::Empty);

                assert_eq!(storage.get_adjacency(corner_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(east_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(northeast_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(north_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(northwest_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(west_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(southwest_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(south_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(southeast_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(south_tile), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(southeast_tile), TileAdjacency::NONE);
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency_multiple_solid_tiles() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(
                    TilePosition::new(layer, IVec2::new(10, 10)),
                    TileMaterial::Wall,
                );
                storage.set_material(
                    TilePosition::new(layer, IVec2::new(11, 10)),
                    TileMaterial::Wall,
                );
                storage.set_material(
                    TilePosition::new(layer, IVec2::new(10, 11)),
                    TileMaterial::Wall,
                );

                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(9, 9))),
                    TileAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(10, 9))),
                    TileAdjacency::NORTH | TileAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(11, 9))),
                    TileAdjacency::NORTH_WEST | TileAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(12, 9))),
                    TileAdjacency::NORTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(9, 10))),
                    TileAdjacency::NORTH_EAST | TileAdjacency::EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(10, 10))),
                    TileAdjacency::NORTH | TileAdjacency::EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(11, 10))),
                    TileAdjacency::NORTH_WEST | TileAdjacency::WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(12, 10))),
                    TileAdjacency::WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(9, 11))),
                    TileAdjacency::EAST | TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(10, 11))),
                    TileAdjacency::SOUTH | TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(11, 11))),
                    TileAdjacency::WEST | TileAdjacency::SOUTH_WEST | TileAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(12, 11))),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(9, 12))),
                    TileAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(10, 12))),
                    TileAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(11, 12))),
                    TileAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_adjacency(TilePosition::new(layer, IVec2::new(12, 12))),
                    TileAdjacency::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_adjacency_overwrite() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer {}).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                let center = TilePosition::new(layer, IVec2::new(5, 5));
                let east_neighbor = TilePosition::new(layer, IVec2::new(6, 5));

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_adjacency(center), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(east_neighbor), TileAdjacency::WEST);

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_adjacency(center), TileAdjacency::NONE);
                assert_eq!(storage.get_adjacency(east_neighbor), TileAdjacency::WEST);
            })
            .unwrap();
    }
}
