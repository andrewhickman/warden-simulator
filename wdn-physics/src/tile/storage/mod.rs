#[cfg(test)]
mod tests;

use std::{fmt, mem};

use bevy_ecs::{
    lifecycle::HookContext,
    prelude::*,
    system::{SystemBuffer, SystemMeta, SystemParam},
    world::DeferredWorld,
};
use bevy_platform::collections::HashMap;

use crate::tile::{
    CHUNK_SIZE_SQUARED,
    adjacency::{Adjacency, TileAdjacency},
    index::TileIndex,
    material::TileMaterial,
    position::{TileChunkOffset, TileChunkPosition, TilePosition},
};

#[derive(SystemParam)]
pub struct TileStorage<'w, 's> {
    map: Res<'w, TileMap>,
    chunks: Query<'w, 's, &'static TileChunk>,
    buffer: Res<'w, TileMapBuffer>,
}

#[derive(SystemParam)]
pub struct TileStorageMut<'w, 's> {
    map: ResMut<'w, TileMap>,
    index: Res<'w, TileIndex>,
    buffer: ResMut<'w, TileMapBuffer>,
    chunks: Query<'w, 's, &'static mut TileChunk>,
    adjacency: Query<'w, 's, &'static mut TileAdjacency>,
    deferred: Deferred<'s, TileStorageDeferred>,
}

#[derive(Component)]
#[component(on_add = TileChunk::on_add, on_remove = TileChunk::on_remove)]
pub struct TileChunk {
    position: TileChunkPosition,
    tiles: Box<[TileData; CHUNK_SIZE_SQUARED]>,
}

#[derive(Default, Resource)]
pub struct TileMap {
    chunks: HashMap<TileChunkPosition, Entity>,
}

#[derive(Default, Resource)]
pub(crate) struct TileMapBuffer {
    chunks: HashMap<TileChunkPosition, TileChunk>,
}

#[derive(Default)]
struct TileStorageDeferred {
    modified: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TileData {
    material: TileMaterial,
    adjacency: TileAdjacency,
}

impl TileStorage<'_, '_> {
    pub fn chunks(&self) -> &Query<'_, '_, &'static TileChunk> {
        &self.chunks
    }

    pub fn contains(&self, tile: TilePosition) -> bool {
        self.map.contains(tile.chunk_position())
    }

    pub fn get(&self, tile: TilePosition) -> Option<&TileData> {
        self.chunk(tile.chunk_position())
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
            Some(t) => t.adjacency(),
            None => TileAdjacency::NONE,
        }
    }

    pub fn get_wall_adjacency(&self, tile: TilePosition) -> Adjacency {
        match self.get(tile) {
            Some(t) => t.wall_adjacency(),
            None => Adjacency::NONE,
        }
    }

    pub fn chunk_id(&self, position: TileChunkPosition) -> Option<Entity> {
        self.map.get(position)
    }

    pub fn chunk(&'_ self, position: TileChunkPosition) -> Option<&TileChunk> {
        if let Some(chunk_id) = self.chunk_id(position) {
            Some(self.chunks.get(chunk_id).expect("invalid chunk entity"))
        } else {
            self.buffer.chunks.get(&position)
        }
    }
}

impl TileStorageMut<'_, '_> {
    pub fn index(&self) -> &TileIndex {
        &self.index
    }

    pub fn get(&self, tile: TilePosition) -> Option<&TileData> {
        self.chunk(tile.chunk_position())
            .map(|chunk| chunk.get(tile.chunk_offset()))
    }

    pub fn get_material(&self, tile: TilePosition) -> TileMaterial {
        match self.get(tile) {
            Some(t) => t.material,
            None => TileMaterial::Empty,
        }
    }

    pub fn get_wall_adjacency(&self, tile: TilePosition) -> Adjacency {
        match self.get(tile) {
            Some(t) => t.wall_adjacency(),
            None => Adjacency::NONE,
        }
    }

    pub fn get_door_adjacency(&self, tile: TilePosition) -> Adjacency {
        match self.get(tile) {
            Some(t) => t.door_adjacency(),
            None => Adjacency::NONE,
        }
    }

    pub fn set_material(&'_ mut self, position: TilePosition, material: TileMaterial) {
        let tile = self
            .chunk_mut(position.chunk_position())
            .get_mut(position.chunk_offset());
        let prev_material = mem::replace(&mut tile.material, material);

        match (prev_material, material) {
            (TileMaterial::Empty, TileMaterial::Wall) => {
                self.add_adjacent_wall(position);
            }
            (TileMaterial::Empty, TileMaterial::Door) => {
                self.add_adjacent_door(position);
            }
            (TileMaterial::Wall, TileMaterial::Empty) => {
                self.remove_adjacent_wall(position);
            }
            (TileMaterial::Wall, TileMaterial::Door) => {
                self.remove_adjacent_wall(position);
                self.add_adjacent_door(position);
            }
            (TileMaterial::Door, TileMaterial::Empty) => {
                self.remove_adjacent_door(position);
            }
            (TileMaterial::Door, TileMaterial::Wall) => {
                self.remove_adjacent_door(position);
                self.add_adjacent_wall(position);
            }
            (TileMaterial::Empty, TileMaterial::Empty)
            | (TileMaterial::Wall, TileMaterial::Wall)
            | (TileMaterial::Door, TileMaterial::Door) => {}
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
            self.deferred.modified = true;
            self.buffer
                .chunks
                .entry(position)
                .or_insert_with(|| TileChunk::empty(position))
        }
    }

    fn add_adjacent_wall(&mut self, position: TilePosition) {
        for (adj, offset) in Adjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.walls.insert(adj);

            if let Some(mut entity_adjacency) = self.get_entity_adjacency(neighbor_pos) {
                entity_adjacency.walls.insert(adj);
            }
        }
    }

    fn remove_adjacent_wall(&mut self, position: TilePosition) {
        for (adj, offset) in Adjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.walls.remove(adj);

            if let Some(mut entity_adjacency) = self.get_entity_adjacency(neighbor_pos) {
                entity_adjacency.walls.remove(adj);
            }
        }
    }

    fn add_adjacent_door(&mut self, position: TilePosition) {
        for (adj, offset) in Adjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.doors.insert(adj);

            if let Some(mut entity_adjacency) = self.get_entity_adjacency(neighbor_pos) {
                entity_adjacency.doors.insert(adj);
            }
        }
    }

    fn remove_adjacent_door(&mut self, position: TilePosition) {
        for (adj, offset) in Adjacency::OFFSETS {
            let neighbor_pos = position.with_offset(offset);

            let neighbour_tile = self
                .chunk_mut(neighbor_pos.chunk_position())
                .get_mut(neighbor_pos.chunk_offset());
            neighbour_tile.adjacency.doors.remove(adj);

            if let Some(mut entity_adjacency) = self.get_entity_adjacency(neighbor_pos) {
                entity_adjacency.doors.remove(adj);
            }
        }
    }

    fn get_entity_adjacency(&mut self, position: TilePosition) -> Option<Mut<'_, TileAdjacency>> {
        if let Some(tile_entity) = self.index.get_tile(position) {
            self.adjacency.get_mut(tile_entity).ok()
        } else {
            None
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
        let prev = self.chunks.insert(position, entity);
        debug_assert!(prev.is_none());
    }

    fn remove(&mut self, position: TileChunkPosition) {
        let prev = self.chunks.remove(&position);
        debug_assert!(prev.is_some());
    }
}

impl TileChunk {
    pub fn empty(position: TileChunkPosition) -> Self {
        Self {
            position,
            tiles: Box::new([TileData::empty(); CHUNK_SIZE_SQUARED]),
        }
    }

    pub fn position(&self) -> TileChunkPosition {
        self.position
    }

    pub fn layer(&self) -> Entity {
        self.position.layer()
    }

    pub fn material(&self, offset: TileChunkOffset) -> TileMaterial {
        self.get(offset).material()
    }

    pub fn adjacency(&self, offset: TileChunkOffset) -> Adjacency {
        self.get(offset).wall_adjacency()
    }

    pub fn get(&self, offset: TileChunkOffset) -> &TileData {
        &self.tiles[offset.index()]
    }

    fn get_mut(&mut self, offset: TileChunkOffset) -> &mut TileData {
        &mut self.tiles[offset.index()]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = (TileChunkOffset, TileData)> {
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
        f.debug_tuple("TileChunk")
            .field(&self.layer())
            .field(&self.position.x())
            .field(&self.position.y())
            .finish_non_exhaustive()
    }
}

impl SystemBuffer for TileStorageDeferred {
    fn queue(&mut self, _system_meta: &SystemMeta, mut world: DeferredWorld) {
        if self.modified {
            let chunks = mem::take(&mut world.resource_mut::<TileMapBuffer>().chunks);
            world
                .commands()
                .spawn_batch(chunks.into_iter().map(|(position, chunk)| {
                    (
                        Name::new(format!("{chunk:?}")),
                        ChildOf(position.layer()),
                        chunk,
                    )
                }));
        }
    }

    fn apply(&mut self, _: &SystemMeta, world: &mut World) {
        if self.modified {
            world.resource_scope(|world: &mut World, mut map: Mut<TileMapBuffer>| {
                if !map.chunks.is_empty() {
                    world.spawn_batch(map.chunks.drain().map(|(position, chunk)| {
                        (
                            Name::new(format!("{chunk:?}")),
                            ChildOf(position.layer()),
                            chunk,
                        )
                    }));
                };
            })
        }
    }
}

impl TileData {
    pub fn empty() -> Self {
        Self {
            material: TileMaterial::Empty,
            adjacency: TileAdjacency::NONE,
        }
    }

    pub fn move_speed(&self) -> f32 {
        self.move_cost().recip()
    }

    pub fn move_cost(&self) -> f32 {
        1.0
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }

    pub fn adjacency(&self) -> TileAdjacency {
        self.adjacency
    }

    pub fn solid_adjacency(&self) -> Adjacency {
        self.adjacency.solid()
    }

    pub fn wall_adjacency(&self) -> Adjacency {
        self.adjacency.walls
    }

    pub fn door_adjacency(&self) -> Adjacency {
        self.adjacency.doors
    }
}
