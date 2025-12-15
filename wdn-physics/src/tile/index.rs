use bevy::{
    platform::collections::{HashMap, hash_map},
    prelude::*,
};
use smallvec::SmallVec;

use crate::tile::TilePosition;

#[derive(Resource, Default, Debug)]
pub struct TileIndex {
    index: HashMap<TilePosition, SmallVec<[Entity; 7]>>,
}

#[derive(Clone, Debug, Message, PartialEq, Eq)]
pub struct TileChanged {
    pub id: Entity,
    pub old: Option<TilePosition>,
    pub new: Option<TilePosition>,
}

impl TileIndex {
    pub fn insert(&mut self, id: Entity, tile: TilePosition) {
        self.index.entry(tile).or_default().push(id);
    }

    pub fn remove(&mut self, id: Entity, tile: TilePosition) {
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

    pub fn get(&self, tile: TilePosition) -> &[Entity] {
        match self.index.get(&tile) {
            Some(entities) => entities,
            None => &[],
        }
    }

    pub fn get_neighborhood(&self, center: TilePosition) -> impl Iterator<Item = Entity> + '_ {
        center
            .neighborhood()
            .into_iter()
            .flat_map(move |tile| self.get(tile).iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::tile::{
        TilePlugin, TilePosition,
        index::{TileChanged, TileIndex},
    };

    #[test]
    fn index_get() {
        let mut index = TileIndex::default();
        let layer = Entity::from_raw_u32(0).unwrap();
        let entity1 = Entity::from_raw_u32(1).unwrap();
        let entity2 = Entity::from_raw_u32(2).unwrap();
        let tile = TilePosition::new(layer, IVec2::new(0, 0));

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
    fn index_get_neighborhood() {
        let mut index = TileIndex::default();
        let layer = Entity::from_raw_u32(0).unwrap();
        let center = TilePosition::new(layer, IVec2::new(1, 1));
        let neighbor = TilePosition::new(layer, IVec2::new(0, 0));
        let far = TilePosition::new(layer, IVec2::new(10, 10));

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

    #[test]
    fn tile_added() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.2, -0.3, 0.0),
                ChildOf(layer),
                TilePosition::default(),
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        let tile = app.world().entity(entity).get::<TilePosition>().unwrap();
        assert_eq!(tile.layer(), layer);
        assert_eq!(tile.position(), IVec2::new(1, -1));

        let index = app.world().resource::<TileIndex>();
        let entities = index.get(TilePosition::new(layer, IVec2::new(1, -1)));
        assert_eq!(entities, &[entity]);

        assert_eq!(
            app.world()
                .resource::<Messages<TileChanged>>()
                .iter_current_update_messages()
                .cloned()
                .collect::<Vec<_>>(),
            vec![TileChanged {
                id: entity,
                old: None,
                new: Some(TilePosition::new(layer, IVec2::new(1, -1))),
            }],
        );
    }

    #[test]
    fn tile_position_changed() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.2, -0.3, 0.0),
                ChildOf(layer),
                TilePosition::default(),
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut()
            .entity_mut(entity)
            .insert(Transform::from_xyz(2.1, -0.2, 0.0));

        app.world_mut().run_schedule(FixedUpdate);

        let tile = app.world().entity(entity).get::<TilePosition>().unwrap();
        assert_eq!(tile.layer(), layer);
        assert_eq!(tile.position(), IVec2::new(2, -1));

        let index = app.world().resource::<TileIndex>();
        let entities = index.get(TilePosition::new(layer, IVec2::new(2, -1)));
        assert_eq!(entities, &[entity]);
        let prev_entities = index.get(TilePosition::new(layer, IVec2::new(1, -1)));
        assert_eq!(prev_entities, &[]);

        assert_eq!(
            app.world()
                .resource::<Messages<TileChanged>>()
                .iter_current_update_messages()
                .cloned()
                .collect::<Vec<_>>(),
            vec![
                TileChanged {
                    id: entity,
                    old: None,
                    new: Some(TilePosition::new(layer, IVec2::new(1, -1))),
                },
                TileChanged {
                    id: entity,
                    old: Some(TilePosition::new(layer, IVec2::new(1, -1))),
                    new: Some(TilePosition::new(layer, IVec2::new(2, -1))),
                }
            ],
        );
    }

    #[test]
    fn tile_layer_changed() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer1 = app.world_mut().spawn_empty().id();
        let layer2 = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(2.3, 1.7, 0.0),
                ChildOf(layer1),
                TilePosition::default(),
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut().entity_mut(entity).insert(ChildOf(layer2));

        app.world_mut().run_schedule(FixedUpdate);

        let tile = app.world().entity(entity).get::<TilePosition>().unwrap();
        assert_eq!(tile.layer(), layer2);
        assert_eq!(tile.position(), IVec2::new(2, 1));

        let index = app.world().resource::<TileIndex>();
        let layer1_entities = index.get(TilePosition::new(layer1, IVec2::new(2, 1)));
        assert_eq!(layer1_entities, &[]);
        let layer2_entities = index.get(TilePosition::new(layer2, IVec2::new(2, 1)));
        assert_eq!(layer2_entities, &[entity]);

        assert_eq!(
            app.world()
                .resource::<Messages<TileChanged>>()
                .iter_current_update_messages()
                .cloned()
                .collect::<Vec<_>>(),
            vec![
                TileChanged {
                    id: entity,
                    old: None,
                    new: Some(TilePosition::new(layer1, IVec2::new(2, 1))),
                },
                TileChanged {
                    id: entity,
                    old: Some(TilePosition::new(layer1, IVec2::new(2, 1))),
                    new: Some(TilePosition::new(layer2, IVec2::new(2, 1))),
                }
            ],
        );
    }

    #[test]
    fn tile_unchanged() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.2, -0.3, 0.0),
                ChildOf(layer),
                TilePosition::default(),
            ))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut()
            .entity_mut(entity)
            .insert(Transform::from_xyz(1.3, -0.2, 0.0));

        let tile_change_tick = app
            .world()
            .entity(entity)
            .get_ref::<TilePosition>()
            .unwrap()
            .last_changed();
        let index_change_tick = app.world().resource_ref::<TileIndex>().last_changed();

        app.world_mut().run_schedule(FixedUpdate);

        let tile = app
            .world()
            .entity(entity)
            .get_ref::<TilePosition>()
            .unwrap();
        assert_eq!(tile.layer(), layer);
        assert_eq!(tile.position(), IVec2::new(1, -1));
        assert_eq!(tile.last_changed(), tile_change_tick);

        let index = app.world().resource_ref::<TileIndex>();
        let entities = index.get(TilePosition::new(layer, IVec2::new(1, -1)));
        assert_eq!(entities, &[entity]);
        assert_eq!(index.last_changed(), index_change_tick);

        assert_eq!(
            app.world()
                .resource::<Messages<TileChanged>>()
                .iter_current_update_messages()
                .cloned()
                .collect::<Vec<_>>(),
            vec![TileChanged {
                id: entity,
                old: None,
                new: Some(TilePosition::new(layer, IVec2::new(1, -1))),
            }],
        );
    }

    #[test]
    fn tile_removed() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.2, -0.3, 0.0),
                ChildOf(layer),
                TilePosition::default(),
            ))
            .id();
        app.world_mut().increment_change_tick();

        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut().entity_mut(entity).despawn();

        app.world_mut().run_schedule(FixedUpdate);

        let index = app.world().resource::<TileIndex>();
        let entities = index.get(TilePosition::new(layer, IVec2::new(1, -1)));
        assert_eq!(entities, &[]);

        assert_eq!(
            app.world()
                .resource::<Messages<TileChanged>>()
                .iter_current_update_messages()
                .cloned()
                .collect::<Vec<_>>(),
            vec![
                TileChanged {
                    id: entity,
                    old: None,
                    new: Some(TilePosition::new(layer, IVec2::new(1, -1))),
                },
                TileChanged {
                    id: entity,
                    old: Some(TilePosition::new(layer, IVec2::new(1, -1))),
                    new: None,
                },
            ],
        );
    }

    #[test]
    fn tile_unset_removed() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((
                Transform::from_xyz(1.2, -0.3, 0.0),
                ChildOf(layer),
                TilePosition::default(),
            ))
            .despawn();

        assert!(app.world().resource::<Messages<TileChanged>>().is_empty());

        let index = app.world().resource::<TileIndex>();
        let entities = index.get(TilePosition::new(layer, IVec2::new(1, -1)));
        assert_eq!(entities, &[]);
    }
}
