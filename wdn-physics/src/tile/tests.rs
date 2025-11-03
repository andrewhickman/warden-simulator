use bevy::prelude::*;

use crate::tile::{
    Tile, TilePlugin,
    index::{TileChanged, TileIndex},
};

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
            Tile::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().entity(entity).get::<Tile>().unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(Tile::new(layer, IVec2::new(1, -1)));
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
            new: Some(Tile::new(layer, IVec2::new(1, -1))),
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
            Tile::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut()
        .entity_mut(entity)
        .insert(Transform::from_xyz(2.1, -0.2, 0.0));

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().entity(entity).get::<Tile>().unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(Tile::new(layer, IVec2::new(2, -1)));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get(Tile::new(layer, IVec2::new(1, -1)));
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
                new: Some(Tile::new(layer, IVec2::new(1, -1))),
            },
            TileChanged {
                id: entity,
                old: Some(Tile::new(layer, IVec2::new(1, -1))),
                new: Some(Tile::new(layer, IVec2::new(2, -1))),
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
            Tile::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().entity_mut(entity).insert(ChildOf(layer2));

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().entity(entity).get::<Tile>().unwrap();
    assert_eq!(tile.layer(), layer2);
    assert_eq!(tile.position(), IVec2::new(2, 1));

    let index = app.world().resource::<TileIndex>();
    let layer1_entities = index.get(Tile::new(layer1, IVec2::new(2, 1)));
    assert_eq!(layer1_entities, &[]);
    let layer2_entities = index.get(Tile::new(layer2, IVec2::new(2, 1)));
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
                new: Some(Tile::new(layer1, IVec2::new(2, 1))),
            },
            TileChanged {
                id: entity,
                old: Some(Tile::new(layer1, IVec2::new(2, 1))),
                new: Some(Tile::new(layer2, IVec2::new(2, 1))),
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
            Tile::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut()
        .entity_mut(entity)
        .insert(Transform::from_xyz(1.3, -0.2, 0.0));

    let tile_change_tick = app
        .world()
        .entity(entity)
        .get_ref::<Tile>()
        .unwrap()
        .last_changed();
    let index_change_tick = app.world().resource_ref::<TileIndex>().last_changed();

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().entity(entity).get_ref::<Tile>().unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));
    assert_eq!(tile.last_changed(), tile_change_tick);

    let index = app.world().resource_ref::<TileIndex>();
    let entities = index.get(Tile::new(layer, IVec2::new(1, -1)));
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
            new: Some(Tile::new(layer, IVec2::new(1, -1))),
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
            Tile::default(),
        ))
        .id();
    app.world_mut().increment_change_tick();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().entity_mut(entity).despawn();

    app.world_mut().run_schedule(FixedUpdate);

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(Tile::new(layer, IVec2::new(1, -1)));
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
                new: Some(Tile::new(layer, IVec2::new(1, -1))),
            },
            TileChanged {
                id: entity,
                old: Some(Tile::new(layer, IVec2::new(1, -1))),
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
            Tile::default(),
        ))
        .despawn();

    assert!(app.world().resource::<Messages<TileChanged>>().is_empty());

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(Tile::new(layer, IVec2::new(1, -1)));
    assert_eq!(entities, &[]);
}
