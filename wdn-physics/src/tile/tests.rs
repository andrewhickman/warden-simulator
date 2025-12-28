use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

use crate::tile::{
    TilePlugin, TilePosition,
    index::TileIndex,
    layer::{Layer, LayerPosition},
};

#[test]
fn tile_position_added() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((ChildOf(layer), TilePosition::new(layer, 1, -1)))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
}

#[test]
fn tile_position_changed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((ChildOf(layer), TilePosition::new(layer, 1, -1)))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut()
        .entity_mut(entity)
        .insert(TilePosition::new(layer, 2, -1));

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 2, -1));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(prev_entities, &[]);
}

#[test]
fn transform_added() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let layer_pos = app.world().get::<LayerPosition>(entity).unwrap();
    assert_eq!(layer_pos.0, Vec2::new(1.2, -0.3));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
}

#[test]
fn transform_changed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut()
        .entity_mut(entity)
        .insert(Transform::from_xyz(2.1, -0.2, 0.0));

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let layer_pos = app.world().get::<LayerPosition>(entity).unwrap();
    assert_eq!(layer_pos.0, Vec2::new(2.1, -0.2));

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 2, -1));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(prev_entities, &[]);
}

#[test]
fn tile_layer_changed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer1 = app.world_mut().spawn(Layer::default()).id();
    let layer2 = app.world_mut().spawn(Layer::default()).id();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.3, 1.7, 0.0),
            ChildOf(layer1),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().entity_mut(entity).insert(ChildOf(layer2));

    app.world_mut().run_schedule(FixedUpdate);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer2);
    assert_eq!(tile.position(), IVec2::new(2, 1));

    let layer_pos = app.world().get::<LayerPosition>(entity).unwrap();
    assert_eq!(layer_pos.0, Vec2::new(2.3, 1.7));

    let index = app.world().resource::<TileIndex>();
    let layer1_entities = index.get(TilePosition::new(layer1, 2, 1));
    assert_eq!(layer1_entities, &[]);
    let layer2_entities = index.get(TilePosition::new(layer2, 2, 1));
    assert_eq!(layer2_entities, &[entity]);
}

#[test]
fn tile_unchanged() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
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

    let layer_pos = app.world().get::<LayerPosition>(entity).unwrap();
    assert_eq!(layer_pos.0, Vec2::new(1.3, -0.2));

    let index = app.world().resource_ref::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
    assert_eq!(index.last_changed(), index_change_tick);
}

#[test]
fn tile_removed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();
    app.world_mut().increment_change_tick();

    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().entity_mut(entity).despawn();

    app.world_mut().run_schedule(FixedUpdate);

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[]);
}

#[test]
fn parent_transform_changed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 3.0, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.5, 0.5, 0.0),
            ChildOf(parent),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(2, 3));

    let parent_layer_pos = app.world().get::<LayerPosition>(parent).unwrap();
    assert_eq!(parent_layer_pos.0, Vec2::new(2.0, 3.0));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(3, 3));

    let child_layer_pos = app.world().get::<LayerPosition>(child).unwrap();
    assert_eq!(child_layer_pos.0, Vec2::new(3.5, 3.5));

    app.world_mut()
        .entity_mut(parent)
        .insert(Transform::from_xyz(4.0, 1.0, 0.0));

    app.world_mut().run_schedule(FixedUpdate);

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(4, 1));

    let parent_layer_pos = app.world().get::<LayerPosition>(parent).unwrap();
    assert_eq!(parent_layer_pos.0, Vec2::new(4.0, 1.0));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(5, 1));

    let child_layer_pos = app.world().get::<LayerPosition>(child).unwrap();
    assert_eq!(child_layer_pos.0, Vec2::new(5.5, 1.5));

    let index = app.world().resource::<TileIndex>();
    let parent_entities = index.get(TilePosition::new(layer, 4, 1));
    assert_eq!(parent_entities, &[parent]);
    let child_entities = index.get(TilePosition::new(layer, 5, 1));
    assert_eq!(child_entities, &[child]);
}

#[test]
fn tile_unset_removed() {
    let mut app = App::new();
    app.add_plugins(TilePlugin);

    let layer = app.world_mut().spawn(Layer::default()).id();
    app.world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::default(),
            LayerPosition::default(),
        ))
        .despawn();

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[]);
}
