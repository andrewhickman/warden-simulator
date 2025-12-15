use std::fmt;

use bevy::{
    ecs::{
        lifecycle::HookContext,
        system::{SystemBuffer, SystemMeta, SystemParam},
        world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};

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
    tiles: Box<[TileMaterial; CHUNK_SIZE * CHUNK_SIZE]>,
}

#[derive(Default, Resource)]
pub struct TileMap {
    chunks: HashMap<(Entity, TileChunkPosition), Entity>,
}

#[derive(Default)]
pub struct TileStorageBuffer {
    chunks: HashMap<(Entity, TileChunkPosition), TileChunk>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileMaterial {
    #[default]
    Empty,
    Wall,
}

impl TileStorage<'_, '_> {
    pub fn material(&self, tile: TilePosition) -> Option<TileMaterial> {
        self.chunk(tile.layer(), tile.chunk_position())
            .map(|chunk| chunk.material(tile.chunk_offset()))
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
    pub fn material(&self, tile: TilePosition) -> Option<TileMaterial> {
        self.chunk(tile.layer(), tile.chunk_position())
            .map(|chunk| chunk.material(tile.chunk_offset()))
    }

    pub fn material_mut(&'_ mut self, tile: TilePosition) -> &mut TileMaterial {
        self.chunk_mut(tile.layer(), tile.chunk_position())
            .material_mut(tile.chunk_offset())
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

    pub fn chunk_mut(&'_ mut self, layer: Entity, position: TileChunkPosition) -> &mut TileChunk {
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
}

impl TileChunk {
    pub fn empty(position: TileChunkPosition) -> Self {
        Self {
            position,
            tiles: Box::new([TileMaterial::Empty; CHUNK_SIZE * CHUNK_SIZE]),
        }
    }

    pub fn position(&self) -> TileChunkPosition {
        self.position
    }

    pub fn material(&self, offset: TileChunkOffset) -> TileMaterial {
        self.tiles[offset.index()]
    }

    pub fn material_mut(&mut self, offset: TileChunkOffset) -> &mut TileMaterial {
        &mut self.tiles[offset.index()]
    }

    pub fn tiles(&self) -> impl ExactSizeIterator<Item = TileMaterial> {
        self.tiles.iter().copied()
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

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::RunSystemOnce,
        math::{I16Vec2, IVec2},
        prelude::*,
    };

    use crate::tile::{
        CHUNK_SIZE, TileChunkOffset, TileChunkPosition, TilePlugin, TilePosition,
        storage::{TileChunk, TileLayer, TileMap, TileMaterial, TileStorage, TileStorageMut},
    };

    #[test]
    fn tile_chunk_empty() {
        let position = TileChunkPosition(I16Vec2::ZERO);
        let chunk = TileChunk::empty(position);

        assert_eq!(chunk.position(), position);
        assert_eq!(chunk.tiles().len(), CHUNK_SIZE * CHUNK_SIZE);

        for material in chunk.tiles() {
            assert_eq!(material, TileMaterial::Empty);
        }
    }

    #[test]
    fn tile_chunk_material_access() {
        let position = TileChunkPosition(I16Vec2::ZERO);
        let mut chunk = TileChunk::empty(position);
        let offset = TileChunkOffset::new(10, 1);

        assert_eq!(chunk.material(offset), TileMaterial::Empty);

        *chunk.material_mut(offset) = TileMaterial::Wall;
        assert_eq!(chunk.material(offset), TileMaterial::Wall);
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
                assert_eq!(storage.material(tile_pos), None);
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
                *storage.material_mut(tile_pos) = TileMaterial::Wall;
                assert_eq!(storage.material(tile_pos), Some(TileMaterial::Wall));
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let tile_pos = TilePosition::new(layer, IVec2::new(5, 7));
                assert_eq!(storage.material(tile_pos), Some(TileMaterial::Wall));
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
                for tile in &range {
                    *storage.material_mut(*tile) = TileMaterial::Wall;
                }

                for tile in &range {
                    assert_eq!(storage.material(*tile), Some(TileMaterial::Wall));
                }
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                for tile in &range_clone {
                    assert_eq!(storage.material(*tile), Some(TileMaterial::Wall));
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

                *storage.material_mut(tile1) = TileMaterial::Wall;
                *storage.material_mut(tile2) = TileMaterial::Empty;

                assert_eq!(storage.material(tile1), Some(TileMaterial::Wall));
                assert_eq!(storage.material(tile2), Some(TileMaterial::Empty));
            })
            .unwrap();

        app.world_mut()
            .run_system_once(move |storage: TileStorage| {
                let position = IVec2::new(10, 15);
                let tile1 = TilePosition::new(layer1, position);
                let tile2 = TilePosition::new(layer2, position);

                assert_eq!(storage.material(tile1), Some(TileMaterial::Wall));
                assert_eq!(storage.material(tile2), Some(TileMaterial::Empty));
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
                assert_eq!(storage.material(tile_pos), None);
            })
            .unwrap();
    }
}
