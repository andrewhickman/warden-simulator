use std::{fmt, mem};

use bevy_ecs::{
    lifecycle::HookContext,
    prelude::*,
    system::{SystemBuffer, SystemMeta, SystemParam},
    world::DeferredWorld,
};
use bevy_math::{CompassOctant, prelude::*};
use bevy_platform::collections::HashMap;
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
#[component(on_add = TileChunk::on_add, on_remove = TileChunk::on_remove)]
pub struct TileChunk {
    position: TileChunkPosition,
    tiles: Box<[Tile; CHUNK_SIZE * CHUNK_SIZE]>,
}

#[derive(Default, Resource)]
pub struct TileMap {
    chunks: HashMap<TileChunkPosition, Entity>,
}

#[derive(Default)]
pub struct TileStorageBuffer {
    chunks: HashMap<TileChunkPosition, TileChunk>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Tile {
    material: TileMaterial,
    occupancy: TileOccupancy,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileOccupancy : u8 {
        const NONE = 0b0000_0000;
        const NORTH = 0b0000_0001;
        const NORTH_EAST = 0b0000_0010;
        const EAST = 0b0000_0100;
        const SOUTH_EAST = 0b0000_1000;
        const SOUTH = 0b0001_0000;
        const SOUTH_WEST = 0b0010_0000;
        const WEST = 0b0100_0000;
        const NORTH_WEST = 0b1000_0000;
    }
}

impl TileStorage<'_, '_> {
    pub fn contains(&self, tile: TilePosition) -> bool {
        self.map.contains(tile.chunk_position())
    }

    pub fn get(&self, tile: TilePosition) -> Option<&Tile> {
        self.chunk(tile.chunk_position())
            .map(|chunk| chunk.get(tile.chunk_offset()))
    }

    pub fn get_material(&self, tile: TilePosition) -> TileMaterial {
        match self.get(tile) {
            Some(t) => t.material,
            None => TileMaterial::Empty,
        }
    }

    pub fn get_occupancy(&self, tile: TilePosition) -> TileOccupancy {
        match self.get(tile) {
            Some(t) => t.occupancy,
            None => TileOccupancy::NONE,
        }
    }

    pub fn chunk_id(&self, position: TileChunkPosition) -> Option<Entity> {
        self.map.get(position)
    }

    pub fn chunk(&'_ self, position: TileChunkPosition) -> Option<&TileChunk> {
        if let Some(chunk_id) = self.chunk_id(position) {
            Some(self.chunks.get(chunk_id).expect("invalid chunk entity"))
        } else {
            None
        }
    }
}

impl TileStorageMut<'_, '_> {
    pub fn get(&self, tile: TilePosition) -> Option<&Tile> {
        self.chunk(tile.chunk_position())
            .map(|chunk| chunk.get(tile.chunk_offset()))
    }

    pub fn get_material(&self, tile: TilePosition) -> TileMaterial {
        match self.get(tile) {
            Some(t) => t.material,
            None => TileMaterial::Empty,
        }
    }

    pub fn get_occupancy(&self, tile: TilePosition) -> TileOccupancy {
        match self.get(tile) {
            Some(t) => t.occupancy,
            None => TileOccupancy::NONE,
        }
    }

    pub fn set_material(&'_ mut self, position: TilePosition, material: TileMaterial) {
        let tile = self
            .chunk_mut(position.chunk_position())
            .get_mut(position.chunk_offset());
        let prev_material = mem::replace(&mut tile.material, material);

        match (prev_material.is_solid(), material.is_solid()) {
            (false, true) => self.add_adjacent(position),
            (true, false) => self.remove_adjacent(position),
            _ => {}
        }
    }

    pub fn chunk(&'_ self, position: TileChunkPosition) -> Option<&TileChunk> {
        if let Some(chunk_entity) = self.map.get(position) {
            Some(self.chunks.get(chunk_entity).expect("invalid chunk entity"))
        } else {
            self.buffer.chunks.get(&position)
        }
    }

    fn chunk_mut(&'_ mut self, position: TileChunkPosition) -> &mut TileChunk {
        if let Some(chunk_entity) = self.map.get(position) {
            self.chunks
                .get_mut(chunk_entity)
                .expect("invalid chunk entity")
                .into_inner()
        } else {
            self.buffer
                .chunks
                .entry(position)
                .or_insert_with(|| TileChunk::empty(position))
        }
    }

    fn add_adjacent(&mut self, position: TilePosition) {
        for (adj, offset) in TileOccupancy::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.occupancy.insert(adj);
        }
    }

    fn remove_adjacent(&mut self, position: TilePosition) {
        for (adj, offset) in TileOccupancy::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.occupancy.remove(adj);
        }
    }
}

impl TileMap {
    pub fn contains(&self, position: TileChunkPosition) -> bool {
        self.chunks.contains_key(&position)
    }

    pub fn get(&self, position: TileChunkPosition) -> Option<Entity> {
        self.chunks.get(&position).copied()
    }

    fn insert(&mut self, position: TileChunkPosition, entity: Entity) {
        self.chunks.insert(position, entity);
    }

    fn remove(&mut self, position: TileChunkPosition) {
        self.chunks.remove(&position);
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

    fn get_mut(&mut self, offset: TileChunkOffset) -> &mut Tile {
        &mut self.tiles[offset.index()]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = &Tile> {
        self.tiles.iter()
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let chunk = world.get::<TileChunk>(context.entity).unwrap().position();
        world
            .resource_mut::<TileMap>()
            .insert(chunk, context.entity);
    }

    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let chunk = world.get::<TileChunk>(context.entity).unwrap().position();
        world.resource_mut::<TileMap>().remove(chunk);
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
                    .map(|(position, chunk)| (ChildOf(position.layer), chunk)),
            );
        }
    }
}

impl Tile {
    pub fn empty() -> Self {
        Self {
            material: TileMaterial::Empty,
            occupancy: TileOccupancy::NONE,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.material.is_solid()
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }

    pub fn occupancy(&self) -> TileOccupancy {
        self.occupancy
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

impl TileOccupancy {
    const OFFSETS: [(TileOccupancy, IVec2); 8] = [
        (TileOccupancy::NORTH, IVec2::new(0, -1)),
        (TileOccupancy::NORTH_EAST, IVec2::new(-1, -1)),
        (TileOccupancy::EAST, IVec2::new(-1, 0)),
        (TileOccupancy::SOUTH_EAST, IVec2::new(-1, 1)),
        (TileOccupancy::SOUTH, IVec2::new(0, 1)),
        (TileOccupancy::SOUTH_WEST, IVec2::new(1, 1)),
        (TileOccupancy::WEST, IVec2::new(1, 0)),
        (TileOccupancy::NORTH_WEST, IVec2::new(1, -1)),
    ];

    pub fn from_octant(octant: CompassOctant) -> Self {
        match octant {
            CompassOctant::North => TileOccupancy::NORTH,
            CompassOctant::NorthEast => TileOccupancy::NORTH_EAST,
            CompassOctant::East => TileOccupancy::EAST,
            CompassOctant::SouthEast => TileOccupancy::SOUTH_EAST,
            CompassOctant::South => TileOccupancy::SOUTH,
            CompassOctant::SouthWest => TileOccupancy::SOUTH_WEST,
            CompassOctant::West => TileOccupancy::WEST,
            CompassOctant::NorthWest => TileOccupancy::NORTH_WEST,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_app::prelude::*;
    use bevy_ecs::{prelude::*, system::RunSystemOnce};
    use bevy_math::{I16Vec2, prelude::*};

    use crate::{
        layer::Layer,
        tile::{
            CHUNK_SIZE, TileChunkOffset, TileChunkPosition, TilePlugin, TilePosition,
            storage::{
                TileChunk, TileMap, TileMaterial, TileOccupancy, TileStorage, TileStorageMut,
            },
        },
    };

    #[test]
    fn tile_chunk_empty() {
        let position = TileChunkPosition {
            layer: Entity::PLACEHOLDER,
            position: I16Vec2::ZERO,
        };
        let chunk = TileChunk::empty(position);

        assert_eq!(chunk.position(), position);
        assert_eq!(chunk.tiles().len(), CHUNK_SIZE * CHUNK_SIZE);

        for tile in chunk.tiles() {
            assert_eq!(tile.material, TileMaterial::Empty);
        }
    }

    #[test]
    fn tile_chunk_material_access() {
        let position = TileChunkPosition {
            layer: Entity::PLACEHOLDER,
            position: I16Vec2::ZERO,
        };
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

        let layer = app.world_mut().spawn(Layer::default()).id();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, 0, 0);
                assert!(storage.get(tile_pos).is_none());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Empty);
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
                storage.set_material(tile_pos, TileMaterial::Wall);
                assert!(storage.get(tile_pos).is_some());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Wall);
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, 5, 7);
                assert!(storage.get(tile_pos).is_some());
                assert_eq!(storage.get_material(tile_pos), TileMaterial::Wall);
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

        let layer1 = app.world_mut().spawn(Layer::default()).id();
        let layer2 = app.world_mut().spawn(Layer::default()).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                let position = IVec2::new(10, 15);
                let tile1 = TilePosition::from_vec(layer1, position);
                let tile2 = TilePosition::from_vec(layer2, position);

                storage.set_material(tile1, TileMaterial::Wall);
                storage.set_material(tile2, TileMaterial::Empty);

                assert_eq!(storage.get_material(tile1), TileMaterial::Wall);
                assert_eq!(storage.get_material(tile2), TileMaterial::Empty);
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let position = IVec2::new(10, 15);
                let tile1 = TilePosition::from_vec(layer1, position);
                let tile2 = TilePosition::from_vec(layer2, position);

                assert_eq!(storage.get_material(tile1), TileMaterial::Wall);
                assert_eq!(storage.get_material(tile2), TileMaterial::Empty);
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
    fn tile_occupancy() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();

        let center = TilePosition::new(layer, 5, 5);

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(center, TileMaterial::Wall);

                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 5)),
                    TileOccupancy::WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 6)),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 6)),
                    TileOccupancy::SOUTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 6)),
                    TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 5)),
                    TileOccupancy::EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 4)),
                    TileOccupancy::NORTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 4)),
                    TileOccupancy::NORTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 4)),
                    TileOccupancy::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 5)),
                    TileOccupancy::WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 6)),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 6)),
                    TileOccupancy::SOUTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 6)),
                    TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 5)),
                    TileOccupancy::EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 4)),
                    TileOccupancy::NORTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 4)),
                    TileOccupancy::NORTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 4)),
                    TileOccupancy::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(center, TileMaterial::Empty);

                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 4)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 4)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 4)),
                    TileOccupancy::NONE
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 6)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 5)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 4, 4)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 5, 4)),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 6, 4)),
                    TileOccupancy::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_occupancy_chunk_edge_horizontal() {
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
                storage.set_material(edge_tile, TileMaterial::Wall);

                assert_eq!(storage.get_occupancy(east_neighbor), TileOccupancy::WEST);
                assert_eq!(storage.get_occupancy(west_neighbor), TileOccupancy::EAST);
                assert_eq!(
                    storage.get_occupancy(northeast_neighbor),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(southeast_neighbor),
                    TileOccupancy::NORTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);
                assert_eq!(storage.get_occupancy(east_neighbor), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(west_neighbor), TileOccupancy::NONE);
                assert_eq!(
                    storage.get_occupancy(northeast_neighbor),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(southeast_neighbor),
                    TileOccupancy::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_occupancy_chunk_edge_vertical() {
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
                storage.set_material(edge_tile, TileMaterial::Wall);

                assert_eq!(storage.get_occupancy(north_neighbor), TileOccupancy::SOUTH);
                assert_eq!(storage.get_occupancy(south_neighbor), TileOccupancy::NORTH);
                assert_eq!(
                    storage.get_occupancy(northwest_neighbor),
                    TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(northeast_neighbor),
                    TileOccupancy::SOUTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);

                assert_eq!(storage.get_occupancy(north_neighbor), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(south_neighbor), TileOccupancy::NONE);
                assert_eq!(
                    storage.get_occupancy(northwest_neighbor),
                    TileOccupancy::NONE
                );
                assert_eq!(
                    storage.get_occupancy(northeast_neighbor),
                    TileOccupancy::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_occupancy_chunk_corner() {
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
                storage.set_material(corner_tile, TileMaterial::Wall);

                assert_eq!(storage.get_occupancy(corner_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(east_tile), TileOccupancy::WEST);
                assert_eq!(
                    storage.get_occupancy(northeast_tile),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(storage.get_occupancy(north_tile), TileOccupancy::SOUTH);
                assert_eq!(
                    storage.get_occupancy(northwest_tile),
                    TileOccupancy::SOUTH_EAST
                );
                assert_eq!(storage.get_occupancy(west_tile), TileOccupancy::EAST);
                assert_eq!(
                    storage.get_occupancy(southwest_tile),
                    TileOccupancy::NORTH_EAST
                );
                assert_eq!(storage.get_occupancy(south_tile), TileOccupancy::NORTH);
                assert_eq!(
                    storage.get_occupancy(southeast_tile),
                    TileOccupancy::NORTH_WEST
                );
                assert_eq!(storage.get_occupancy(south_tile), TileOccupancy::NORTH);
                assert_eq!(
                    storage.get_occupancy(southeast_tile),
                    TileOccupancy::NORTH_WEST
                );

                storage.set_material(corner_tile, TileMaterial::Empty);

                assert_eq!(storage.get_occupancy(corner_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(east_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(northeast_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(north_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(northwest_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(west_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(southwest_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(south_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(southeast_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(south_tile), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(southeast_tile), TileOccupancy::NONE);
            })
            .unwrap();
    }

    #[test]
    fn tile_occupancy_multiple_solid_tiles() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(TilePosition::new(layer, 10, 10), TileMaterial::Wall);
                storage.set_material(TilePosition::new(layer, 11, 10), TileMaterial::Wall);
                storage.set_material(TilePosition::new(layer, 10, 11), TileMaterial::Wall);

                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 9, 9)),
                    TileOccupancy::NORTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 10, 9)),
                    TileOccupancy::NORTH | TileOccupancy::NORTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 11, 9)),
                    TileOccupancy::NORTH_WEST | TileOccupancy::NORTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 12, 9)),
                    TileOccupancy::NORTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 9, 10)),
                    TileOccupancy::NORTH_EAST | TileOccupancy::EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 10, 10)),
                    TileOccupancy::NORTH | TileOccupancy::EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 11, 10)),
                    TileOccupancy::NORTH_WEST | TileOccupancy::WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 12, 10)),
                    TileOccupancy::WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 9, 11)),
                    TileOccupancy::EAST | TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 10, 11)),
                    TileOccupancy::SOUTH | TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 11, 11)),
                    TileOccupancy::WEST | TileOccupancy::SOUTH_WEST | TileOccupancy::SOUTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 12, 11)),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 9, 12)),
                    TileOccupancy::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 10, 12)),
                    TileOccupancy::SOUTH
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 11, 12)),
                    TileOccupancy::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_occupancy(TilePosition::new(layer, 12, 12)),
                    TileOccupancy::NONE
                );
            })
            .unwrap();
    }

    #[test]
    fn tile_occupancy_overwrite() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                let center = TilePosition::new(layer, 5, 5);
                let east_neighbor = TilePosition::new(layer, 6, 5);

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_occupancy(center), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(east_neighbor), TileOccupancy::WEST);

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_occupancy(center), TileOccupancy::NONE);
                assert_eq!(storage.get_occupancy(east_neighbor), TileOccupancy::WEST);
            })
            .unwrap();
    }
}
