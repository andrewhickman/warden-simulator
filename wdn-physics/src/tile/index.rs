use bevy::{
    platform::collections::{HashMap, hash_map},
    prelude::*,
};
use smallvec::SmallVec;

use crate::tile::Tile;

#[derive(Resource, Default, Debug)]
pub struct TileIndex {
    index: HashMap<Tile, SmallVec<[Entity; 7]>>,
}

#[derive(Clone, Debug, Message, PartialEq, Eq)]
pub struct TileChanged {
    pub id: Entity,
    pub old: Option<Tile>,
    pub new: Option<Tile>,
}

impl TileIndex {
    pub fn insert(&mut self, id: Entity, tile: Tile) {
        self.index.entry(tile).or_default().push(id);
    }

    pub fn remove(&mut self, id: Entity, tile: Tile) {
        if let hash_map::Entry::Occupied(mut entry) = self.index.entry(tile) {
            let entities = entry.get_mut();
            if let Some(pos) = entities.iter().position(|&e| e == id) {
                entities.swap_remove(pos);
            }
            if entities.is_empty() {
                entry.remove();
            }
        }
    }

    pub fn get(&self, tile: Tile) -> &[Entity] {
        match self.index.get(&tile) {
            Some(entities) => entities,
            None => &[],
        }
    }

    pub fn get_neighborhood(&self, center: Tile) -> impl Iterator<Item = Entity> + '_ {
        center
            .neighborhood()
            .into_iter()
            .flat_map(move |tile| self.get(tile).iter().copied())
    }
}

#[test]
fn test_index_get() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let entity1 = Entity::from_raw_u32(1).unwrap();
    let entity2 = Entity::from_raw_u32(2).unwrap();
    let tile = Tile::new(layer, IVec2::new(0, 0));

    index.insert(entity1, tile);
    index.insert(entity2, tile);
    assert_eq!(index.get(tile).len(), 2);
    assert!(index.get(tile).contains(&entity1));
    assert!(index.get(tile).contains(&entity2));

    index.remove(entity1, tile);
    assert_eq!(index.get(tile), &[entity2]);

    index.remove(entity2, tile);
    assert_eq!(index.get(tile), &[]);
    assert!(!index.index.contains_key(&tile));

    index.remove(entity1, tile);
    assert_eq!(index.get(tile), &[]);
}

#[test]
fn test_index_get_neighborhood() {
    let mut index = TileIndex::default();
    let layer = Entity::from_raw_u32(0).unwrap();
    let center = Tile::new(layer, IVec2::new(1, 1));
    let neighbor = Tile::new(layer, IVec2::new(0, 0));
    let far = Tile::new(layer, IVec2::new(10, 10));

    let entity1 = Entity::from_raw_u32(1).unwrap();
    let entity2 = Entity::from_raw_u32(2).unwrap();
    let entity3 = Entity::from_raw_u32(3).unwrap();

    index.insert(entity1, center);
    index.insert(entity2, neighbor);
    index.insert(entity3, far);

    let neighborhood: Vec<Entity> = index.get_neighborhood(center).collect();
    assert!(neighborhood.contains(&entity1));
    assert!(neighborhood.contains(&entity2));
    assert!(!neighborhood.contains(&entity3));
}
