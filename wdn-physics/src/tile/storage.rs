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
    wall_adjacency: WallAdjacency,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WallAdjacency : u8 {
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

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DoorAdjacency : u8 {
        const NONE = 0b0000_0000;
        const NORTH = 0b0000_0001;
        const EAST = 0b0000_0010;
        const SOUTH = 0b0000_0100;
        const WEST = 0b0000_1000;
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

    pub fn get_wall_adjacency(&self, tile: TilePosition) -> WallAdjacency {
        match self.get(tile) {
            Some(t) => t.wall_adjacency,
            None => WallAdjacency::NONE,
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

    pub fn get_wall_adjacency(&self, tile: TilePosition) -> WallAdjacency {
        match self.get(tile) {
            Some(t) => t.wall_adjacency,
            None => WallAdjacency::NONE,
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
        for (adj, offset) in WallAdjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.wall_adjacency.insert(adj);
        }
    }

    fn remove_adjacent(&mut self, position: TilePosition) {
        for (adj, offset) in WallAdjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.wall_adjacency.remove(adj);
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

    pub fn layer(&self) -> Entity {
        self.position.layer
    }

    pub fn is_solid(&self, offset: TileChunkOffset) -> bool {
        self.get(offset).is_solid()
    }

    pub fn adjacency(&self, offset: TileChunkOffset) -> WallAdjacency {
        self.get(offset).wall_adjacency
    }

    pub fn get(&self, offset: TileChunkOffset) -> &Tile {
        &self.tiles[offset.index()]
    }

    fn get_mut(&mut self, offset: TileChunkOffset) -> &mut Tile {
        &mut self.tiles[offset.index()]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = (TileChunkOffset, Tile)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(i, &tile)| (TileChunkOffset::from_index(i), tile))
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
            wall_adjacency: WallAdjacency::NONE,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.material.is_solid()
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }

    pub fn wall_adjacency(&self) -> WallAdjacency {
        self.wall_adjacency
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

impl WallAdjacency {
    const OFFSETS: [(WallAdjacency, IVec2); 8] = [
        (WallAdjacency::NORTH, IVec2::new(0, -1)),
        (WallAdjacency::NORTH_EAST, IVec2::new(-1, -1)),
        (WallAdjacency::EAST, IVec2::new(-1, 0)),
        (WallAdjacency::SOUTH_EAST, IVec2::new(-1, 1)),
        (WallAdjacency::SOUTH, IVec2::new(0, 1)),
        (WallAdjacency::SOUTH_WEST, IVec2::new(1, 1)),
        (WallAdjacency::WEST, IVec2::new(1, 0)),
        (WallAdjacency::NORTH_WEST, IVec2::new(1, -1)),
    ];

    pub fn from_octant(octant: CompassOctant) -> Self {
        match octant {
            CompassOctant::North => WallAdjacency::NORTH,
            CompassOctant::NorthEast => WallAdjacency::NORTH_EAST,
            CompassOctant::East => WallAdjacency::EAST,
            CompassOctant::SouthEast => WallAdjacency::SOUTH_EAST,
            CompassOctant::South => WallAdjacency::SOUTH,
            CompassOctant::SouthWest => WallAdjacency::SOUTH_WEST,
            CompassOctant::West => WallAdjacency::WEST,
            CompassOctant::NorthWest => WallAdjacency::NORTH_WEST,
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
                TileChunk, TileMap, TileMaterial, TileStorage, TileStorageMut, WallAdjacency,
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

        for (_, tile) in chunk.tiles() {
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
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                    WallAdjacency::WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                    WallAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                    WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                    WallAdjacency::EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                    WallAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                    WallAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                    WallAdjacency::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                    WallAdjacency::WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                    WallAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                    WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                    WallAdjacency::EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                    WallAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                    WallAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                    WallAdjacency::NORTH_WEST
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |mut storage: TileStorageMut| {
                storage.set_material(center, TileMaterial::Empty);

                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                    WallAdjacency::NONE
                );
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 6)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 5)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 4, 4)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 5, 4)),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 6, 4)),
                    WallAdjacency::NONE
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

                assert_eq!(
                    storage.get_wall_adjacency(east_neighbor),
                    WallAdjacency::WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(west_neighbor),
                    WallAdjacency::EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(northeast_neighbor),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(southeast_neighbor),
                    WallAdjacency::NORTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);
                assert_eq!(
                    storage.get_wall_adjacency(east_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(west_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(northeast_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(southeast_neighbor),
                    WallAdjacency::NONE
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

                assert_eq!(
                    storage.get_wall_adjacency(north_neighbor),
                    WallAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(south_neighbor),
                    WallAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(northwest_neighbor),
                    WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(northeast_neighbor),
                    WallAdjacency::SOUTH_WEST
                );

                storage.set_material(edge_tile, TileMaterial::Empty);

                assert_eq!(
                    storage.get_wall_adjacency(north_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(south_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(northwest_neighbor),
                    WallAdjacency::NONE
                );
                assert_eq!(
                    storage.get_wall_adjacency(northeast_neighbor),
                    WallAdjacency::NONE
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

                assert_eq!(storage.get_wall_adjacency(corner_tile), WallAdjacency::NONE);
                assert_eq!(storage.get_wall_adjacency(east_tile), WallAdjacency::WEST);
                assert_eq!(
                    storage.get_wall_adjacency(northeast_tile),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(storage.get_wall_adjacency(north_tile), WallAdjacency::SOUTH);
                assert_eq!(
                    storage.get_wall_adjacency(northwest_tile),
                    WallAdjacency::SOUTH_EAST
                );
                assert_eq!(storage.get_wall_adjacency(west_tile), WallAdjacency::EAST);
                assert_eq!(
                    storage.get_wall_adjacency(southwest_tile),
                    WallAdjacency::NORTH_EAST
                );
                assert_eq!(storage.get_wall_adjacency(south_tile), WallAdjacency::NORTH);
                assert_eq!(
                    storage.get_wall_adjacency(southeast_tile),
                    WallAdjacency::NORTH_WEST
                );
                assert_eq!(storage.get_wall_adjacency(south_tile), WallAdjacency::NORTH);
                assert_eq!(
                    storage.get_wall_adjacency(southeast_tile),
                    WallAdjacency::NORTH_WEST
                );

                storage.set_material(corner_tile, TileMaterial::Empty);

                assert_eq!(storage.get_wall_adjacency(corner_tile), WallAdjacency::NONE);
                assert_eq!(storage.get_wall_adjacency(east_tile), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(northeast_tile),
                    WallAdjacency::NONE
                );
                assert_eq!(storage.get_wall_adjacency(north_tile), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(northwest_tile),
                    WallAdjacency::NONE
                );
                assert_eq!(storage.get_wall_adjacency(west_tile), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(southwest_tile),
                    WallAdjacency::NONE
                );
                assert_eq!(storage.get_wall_adjacency(south_tile), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(southeast_tile),
                    WallAdjacency::NONE
                );
                assert_eq!(storage.get_wall_adjacency(south_tile), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(southeast_tile),
                    WallAdjacency::NONE
                );
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
                    storage.get_wall_adjacency(TilePosition::new(layer, 9, 9)),
                    WallAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 10, 9)),
                    WallAdjacency::NORTH | WallAdjacency::NORTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 11, 9)),
                    WallAdjacency::NORTH_WEST | WallAdjacency::NORTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 12, 9)),
                    WallAdjacency::NORTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 9, 10)),
                    WallAdjacency::NORTH_EAST | WallAdjacency::EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 10, 10)),
                    WallAdjacency::NORTH | WallAdjacency::EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 11, 10)),
                    WallAdjacency::NORTH_WEST | WallAdjacency::WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 12, 10)),
                    WallAdjacency::WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 9, 11)),
                    WallAdjacency::EAST | WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 10, 11)),
                    WallAdjacency::SOUTH | WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 11, 11)),
                    WallAdjacency::WEST | WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 12, 11)),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 9, 12)),
                    WallAdjacency::SOUTH_EAST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 10, 12)),
                    WallAdjacency::SOUTH
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 11, 12)),
                    WallAdjacency::SOUTH_WEST
                );
                assert_eq!(
                    storage.get_wall_adjacency(TilePosition::new(layer, 12, 12)),
                    WallAdjacency::NONE
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
                let east_neighbor = center.east();

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_wall_adjacency(center), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(east_neighbor),
                    WallAdjacency::WEST
                );

                storage.set_material(center, TileMaterial::Wall);
                storage.set_material(east_neighbor, TileMaterial::Empty);

                assert_eq!(storage.get_wall_adjacency(center), WallAdjacency::NONE);
                assert_eq!(
                    storage.get_wall_adjacency(east_neighbor),
                    WallAdjacency::WEST
                );
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
                storage.set_material(tile1, TileMaterial::Wall);
                storage.set_material(tile2, TileMaterial::Wall);
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
                storage.set_material(tile1, TileMaterial::Empty);
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
                storage.set_material(edge, TileMaterial::Wall);
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
                storage.set_material(edge, TileMaterial::Empty);
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
}
