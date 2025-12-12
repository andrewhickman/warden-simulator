use std::fmt;

use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};

use crate::tile::{CHUNK_SIZE, Tile};

#[derive(SystemParam)]
pub struct TileStorage<'w, 's> {
    map: Res<'w, TileMap>,
    layers: Query<'w, 's, &'static TileLayer>,
    chunks: Query<'w, 's, &'static TileChunk>,
}

#[derive(SystemParam)]
pub struct TileStorageMut<'w, 's> {
    map: ResMut<'w, TileMap>,
    layers: Query<'w, 's, &'static mut TileLayer>,
    chunks: Query<'w, 's, &'static mut TileChunk>,
}

#[derive(Resource)]
pub struct TileMap {
    layers: HashMap<Entity, Entity>,
    chunks: HashMap<(Entity, TileChunkOffset), Entity>,
}

#[derive(Component)]
pub struct TileLayer {}

#[derive(Component)]
pub struct TileChunk {
    offset: TileChunkOffset,
    tiles: Box<[TileMaterial; CHUNK_SIZE * CHUNK_SIZE]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileChunkOffset(IVec2);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
}

impl TileStorage<'_, '_> {
    pub fn material(&self, tile: Tile) -> Option<TileMaterial> {
        let &chunk_entity = self.map.chunks.get(&(tile.layer(), tile.chunk()))?;
        let chunk = self.chunks.get(chunk_entity).ok()?;
        Some(chunk.material(tile.position()))
    }
}

impl TileStorageMut<'_, '_> {
    pub fn material(&self, tile: Tile) -> Option<TileMaterial> {
        let &chunk_entity = self.map.chunks.get(&(tile.layer(), tile.chunk()))?;
        let chunk = self.chunks.get(chunk_entity).ok()?;
        Some(chunk.material(tile.position()))
    }

    pub fn material_mut(&'_ mut self, tile: Tile) -> Option<Mut<'_, TileMaterial>> {
        let &chunk_entity = self.map.chunks.get(&(tile.layer(), tile.chunk()))?;
        let chunk = self.chunks.get_mut(chunk_entity).ok()?;
        Some(chunk.map_unchanged(|chunk| chunk.material_mut(tile.position())))
    }
}

impl TileChunk {
    pub fn empty(offset: TileChunkOffset) -> Self {
        Self {
            offset,
            tiles: Box::new([TileMaterial::Empty; CHUNK_SIZE * CHUNK_SIZE]),
        }
    }

    pub fn offset(&self) -> TileChunkOffset {
        self.offset
    }

    pub fn material(&self, position: IVec2) -> TileMaterial {
        let local_x = position.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = position.y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let index = local_y * CHUNK_SIZE + local_x;

        self.tiles[index]
    }

    pub fn material_mut(&mut self, position: IVec2) -> &mut TileMaterial {
        let local_x = position.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = position.y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let index = local_y * CHUNK_SIZE + local_x;

        &mut self.tiles[index]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = TileMaterial> {
        self.tiles.iter().copied()
    }
}

impl fmt::Debug for TileChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileChunk")
            .field("offset", &self.offset)
            .finish_non_exhaustive()
    }
}

impl TileChunkOffset {
    pub fn from_position(position: IVec2) -> Self {
        TileChunkOffset(IVec2::new(
            position.x.div_euclid(CHUNK_SIZE as i32),
            position.y.div_euclid(CHUNK_SIZE as i32),
        ))
    }
}
