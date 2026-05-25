use bevy_ecs::prelude::*;
use bevy_platform::collections::{HashMap, hash_map};
use smallvec::SmallVec;

use crate::tile::position::TilePosition;

#[derive(Resource, Default, Debug)]
pub struct TileIndex {
    index: HashMap<TilePosition, TileIndexValue>,
}

#[derive(Default, Debug)]
struct TileIndexValue {
    objects: SmallVec<[Entity; 4]>,
    tile: Option<Entity>,
}

impl TileIndex {
    pub fn insert_object(&mut self, id: Entity, tile: TilePosition) {
        self.index.entry(tile).or_default().objects.push(id);
    }

    pub fn insert_tile(&mut self, id: Entity, tile: TilePosition) -> Option<Entity> {
        self.index.entry(tile).or_default().tile.replace(id)
    }

    pub fn remove_object(&mut self, id: Entity, tile: TilePosition) {
        if let hash_map::Entry::Occupied(mut entry) = self.index.entry(tile) {
            let entities = entry.get_mut();
            if let Some(pos) = entities.objects.iter().position(|&e| e == id) {
                entities.objects.swap_remove(pos);
            }
            if entities.objects.is_empty() && entities.tile.is_none() {
                entry.remove();
            }
        }
    }

    pub fn remove_tile(&mut self, id: Entity, tile: TilePosition) {
        if let hash_map::Entry::Occupied(mut entry) = self.index.entry(tile) {
            let entities = entry.get_mut();
            if entities.tile == Some(id) {
                entities.tile = None;
            }
            if entities.objects.is_empty() && entities.tile.is_none() {
                entry.remove();
            }
        }
    }

    pub fn get_objects(&self, tile: TilePosition) -> &[Entity] {
        match self.index.get(&tile) {
            Some(entities) => &entities.objects,
            None => &[],
        }
    }

    pub fn get_tile(&self, tile: TilePosition) -> Option<Entity> {
        match self.index.get(&tile) {
            Some(entities) => entities.tile,
            None => None,
        }
    }
}

#[test]
fn test_index_entry_size() {
    assert_eq!(std::mem::size_of::<(TilePosition, TileIndexValue)>(), 64);
}

#[test]
fn test_index_get_objects() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let entity1 = Entity::from_raw_u32(1).unwrap();
    let entity2 = Entity::from_raw_u32(2).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    index.insert_object(entity1, tile);
    index.insert_object(entity2, tile);
    assert_eq!(index.get_objects(tile).len(), 2);
    assert!(index.get_objects(tile).contains(&entity1));
    assert!(index.get_objects(tile).contains(&entity2));

    index.remove_object(entity1, tile);
    assert_eq!(index.get_objects(tile), &[entity2]);

    index.remove_object(entity2, tile);
    assert_eq!(index.get_objects(tile), &[]);
    assert!(!index.index.contains_key(&tile));

    index.remove_object(entity1, tile);
    assert_eq!(index.get_objects(tile), &[]);
}

#[test]
fn test_index_get_tile_empty() {
    let index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    assert_eq!(index.get_tile(tile), None);
}

#[test]
fn test_index_get_tile() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let tile_entity1 = Entity::from_raw_u32(1).unwrap();
    let tile_entity2 = Entity::from_raw_u32(2).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    assert_eq!(index.insert_tile(tile_entity1, tile), None);
    assert_eq!(index.get_tile(tile), Some(tile_entity1));

    assert_eq!(index.insert_tile(tile_entity2, tile), Some(tile_entity1));
    assert_eq!(index.get_tile(tile), Some(tile_entity2));
}

#[test]
fn test_index_remove_tile() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let tile_entity = Entity::from_raw_u32(1).unwrap();
    let wrong_tile_entity = Entity::from_raw_u32(2).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    index.insert_tile(tile_entity, tile);
    index.remove_tile(wrong_tile_entity, tile);
    assert_eq!(index.get_tile(tile), Some(tile_entity));

    index.remove_tile(tile_entity, tile);
    assert_eq!(index.get_tile(tile), None);
    assert!(!index.index.contains_key(&tile));

    index.remove_tile(tile_entity, tile);
    assert_eq!(index.get_tile(tile), None);
}

#[test]
fn test_index_remove_tile_objects_exist() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let tile_entity = Entity::from_raw_u32(1).unwrap();
    let object_entity = Entity::from_raw_u32(2).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    index.insert_tile(tile_entity, tile);
    index.insert_object(object_entity, tile);

    index.remove_tile(tile_entity, tile);
    assert_eq!(index.get_tile(tile), None);
    assert_eq!(index.get_objects(tile), &[object_entity]);
    assert!(index.index.contains_key(&tile));

    index.remove_object(object_entity, tile);
    assert_eq!(index.get_objects(tile), &[]);
    assert!(!index.index.contains_key(&tile));
}

#[test]
fn test_index_remove_object_tile_exists() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let tile_entity = Entity::from_raw_u32(1).unwrap();
    let object_entity = Entity::from_raw_u32(2).unwrap();
    let wrong_object_entity = Entity::from_raw_u32(3).unwrap();
    let tile = TilePosition::new(layer, 0, 0);

    index.insert_tile(tile_entity, tile);
    index.insert_object(object_entity, tile);

    index.remove_object(wrong_object_entity, tile);
    assert_eq!(index.get_objects(tile), &[object_entity]);
    assert_eq!(index.get_tile(tile), Some(tile_entity));

    index.remove_object(object_entity, tile);
    assert_eq!(index.get_objects(tile), &[]);
    assert_eq!(index.get_tile(tile), Some(tile_entity));
    assert!(index.index.contains_key(&tile));

    index.remove_tile(tile_entity, tile);
    assert_eq!(index.get_tile(tile), None);
    assert!(!index.index.contains_key(&tile));
}
