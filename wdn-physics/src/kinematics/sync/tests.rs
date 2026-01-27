use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::prelude::*;
use bevy_time::TimePlugin;

use crate::{
    kinematics::{
        GlobalPosition, GlobalVelocity, KinematicsPlugin, Position, Velocity, sync::sync_kinematics,
    },
    layer::Layer,
    tile::{TilePlugin, TilePosition, index::TileIndex},
};

#[test]
fn sync_kinematics_transform_added() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.2, -0.3), Rot2::radians(FRAC_PI_4)),
            ChildOf(layer),
        ))
        .id();

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let layer_pos = app.world().get::<GlobalPosition>(entity).unwrap();
    assert_relative_eq!(layer_pos.position(), Vec2::new(1.2, -0.3));
    assert_relative_eq!(layer_pos.rotation().as_radians(), FRAC_PI_4);

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
}

#[test]
fn sync_kinematics_transform_changed() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.2, -0.3), Rot2::radians(FRAC_PI_4)),
            ChildOf(layer),
        ))
        .id();

    app.world_mut().entity_mut(entity).insert(Position::new(
        Vec2::new(2.1, -0.2),
        Rot2::radians(-FRAC_PI_2),
    ));

    run_sync_kinematics(&mut app);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let layer_pos = app.world().get::<GlobalPosition>(entity).unwrap();
    assert_relative_eq!(layer_pos.position(), Vec2::new(2.1, -0.2));
    assert_relative_eq!(layer_pos.rotation().as_radians(), -FRAC_PI_2);

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 2, -1));
    assert_eq!(entities, &[entity]);
    let prev_entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(prev_entities, &[]);
}

#[test]
fn sync_kinematics_tile_layer_changed() {
    let mut app = make_app();

    let layer1 = app.world_mut().spawn(Layer::default()).id();
    let layer2 = app.world_mut().spawn(Layer::default()).id();

    let entity = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.3, 1.7), Rot2::IDENTITY),
            ChildOf(layer1),
            TilePosition::new(layer1, 2, 1),
        ))
        .id();

    app.world_mut().entity_mut(entity).insert(ChildOf(layer2));

    run_sync_kinematics(&mut app);

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer2);
    assert_eq!(tile.position(), IVec2::new(2, 1));

    let layer_pos = app.world().get::<GlobalPosition>(entity).unwrap();
    assert_relative_eq!(layer_pos.position(), Vec2::new(2.3, 1.7));

    let index = app.world().resource::<TileIndex>();
    let layer1_entities = index.get(TilePosition::new(layer1, 2, 1));
    assert_eq!(layer1_entities, &[]);
    let layer2_entities = index.get(TilePosition::new(layer2, 2, 1));
    assert_eq!(layer2_entities, &[entity]);
}

#[test]
fn sync_kinematics_tile_unchanged() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.2, -0.3), Rot2::IDENTITY),
            ChildOf(layer),
        ))
        .id();

    app.world_mut()
        .entity_mut(entity)
        .insert(Position::new(Vec2::new(1.3, -0.2), Rot2::IDENTITY));

    let tile_change_tick = app
        .world()
        .entity(entity)
        .get_ref::<TilePosition>()
        .unwrap()
        .last_changed();
    let index_change_tick = app.world().resource_ref::<TileIndex>().last_changed();

    run_sync_kinematics(&mut app);

    let tile = app
        .world()
        .entity(entity)
        .get_ref::<TilePosition>()
        .unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));
    assert_eq!(tile.last_changed(), tile_change_tick);

    let layer_pos = app.world().get::<GlobalPosition>(entity).unwrap();
    assert_relative_eq!(layer_pos.position(), Vec2::new(1.3, -0.2));

    let index = app.world().resource_ref::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[entity]);
    assert_eq!(index.last_changed(), index_change_tick);
}

#[test]
fn sync_kinematics_parent_transform_changed() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 3.0), Rot2::radians(FRAC_PI_4)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.5, 0.5), Rot2::radians(FRAC_PI_4)),
            ChildOf(parent),
        ))
        .id();

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(2, 3));

    let parent_layer_pos = app.world().get::<GlobalPosition>(parent).unwrap();
    assert_relative_eq!(parent_layer_pos.position(), Vec2::new(2.0, 3.0));
    assert_relative_eq!(parent_layer_pos.rotation().as_radians(), FRAC_PI_4);

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(2, 4));

    let child_layer_pos = app.world().get::<GlobalPosition>(child).unwrap();
    assert_relative_eq!(
        child_layer_pos.position(),
        Vec2::new(2.707107, 4.414214),
        epsilon = 1e-4
    );
    assert_relative_eq!(
        child_layer_pos.rotation().as_radians(),
        FRAC_PI_2,
        epsilon = 1e-4
    );

    app.world_mut()
        .entity_mut(parent)
        .insert(Position::new(Vec2::new(4.0, 1.0), Rot2::radians(FRAC_PI_2)));

    run_sync_kinematics(&mut app);

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(4, 1));

    let parent_layer_pos = app.world().get::<GlobalPosition>(parent).unwrap();
    assert_relative_eq!(parent_layer_pos.position(), Vec2::new(4.0, 1.0));
    assert_relative_eq!(parent_layer_pos.rotation().as_radians(), FRAC_PI_2);

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(3, 2));

    let child_layer_pos = app.world().get::<GlobalPosition>(child).unwrap();
    assert_relative_eq!(
        child_layer_pos.position(),
        Vec2::new(3.5, 2.5),
        epsilon = 1e-4
    );
    assert_relative_eq!(
        child_layer_pos.rotation().as_radians(),
        FRAC_PI_2 + FRAC_PI_4,
        epsilon = 1e-4
    );

    let index = app.world().resource::<TileIndex>();
    let parent_entities = index.get(TilePosition::new(layer, 4, 1));
    assert_eq!(parent_entities, &[parent]);
    let child_entities = index.get(TilePosition::new(layer, 3, 2));
    assert_eq!(child_entities, &[child]);
}

#[test]
fn sync_kinematics_tile_unset_removed() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    app.world_mut()
        .spawn((
            Position::new(Vec2::new(1.2, -0.3), Rot2::IDENTITY),
            ChildOf(layer),
            TilePosition::new(layer, 1, -1),
        ))
        .despawn();

    let index = app.world().resource::<TileIndex>();
    let entities = index.get(TilePosition::new(layer, 1, -1));
    assert_eq!(entities, &[]);
}

#[test]
fn sync_kinematics_velocity_no_parent() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.0, 2.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(3.0, 4.0)).with_angular(0.5),
            ChildOf(layer),
        ))
        .id();

    let velocity = app.world().get::<GlobalVelocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(3.0, 4.0));
    assert_relative_eq!(velocity.angular(), 0.5);
}

#[test]
fn sync_kinematics_velocity_parent_linear() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
            Velocity::new(Vec2::new(2.0, 1.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.0, 0.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(0.5, 0.5)),
            ChildOf(parent),
        ))
        .id();

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::new(2.0, 1.0));
    assert_relative_eq!(parent_velocity.angular(), 0.0);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(2.5, 1.5), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 0.0);
}

#[test]
fn sync_kinematics_velocity_parent_angular() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
            Velocity::new(Vec2::ZERO).with_angular(1.0),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 0.0), Rot2::IDENTITY),
            Velocity::new(Vec2::ZERO),
            ChildOf(parent),
        ))
        .id();

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::ZERO);
    assert_relative_eq!(parent_velocity.angular(), 1.0);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(0.0, 2.0), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 1.0);
}

#[test]
fn sync_kinematics_velocity_parent_combined() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
            Velocity::new(Vec2::new(3.0, 0.0)).with_angular(0.5),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(0.0, 4.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(1.0, 1.0)).with_angular(0.2),
            ChildOf(parent),
        ))
        .id();

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::new(3.0, 0.0));
    assert_relative_eq!(parent_velocity.angular(), 0.5);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(2.0, 1.0), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 0.7);
}

#[test]
fn sync_kinematics_velocity_parent_rotated() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::radians(FRAC_PI_2)),
            Velocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 0.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(parent),
        ))
        .id();

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.0, 0.0),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.0);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(1.0, 1.0), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 0.0);
}

#[test]
fn sync_kinematics_velocity_grandparent() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let grandparent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
            Velocity::new(Vec2::new(1.0, 0.0)).with_angular(0.1),
            ChildOf(layer),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 0.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(0.0, 1.0)).with_angular(0.2),
            ChildOf(grandparent),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(0.0, 3.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(0.5, 0.5)).with_angular(0.3),
            ChildOf(parent),
        ))
        .id();

    let grandparent_velocity = app.world().get::<GlobalVelocity>(grandparent).unwrap();
    assert_relative_eq!(grandparent_velocity.linear(), Vec2::new(1.0, 0.0));
    assert_relative_eq!(grandparent_velocity.angular(), 0.1);

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.0, 1.2),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.3);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(0.6, 1.7), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 0.6);
}

#[test]
fn sync_kinematics_velocity_updated_on_change() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
            Velocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 0.0), Rot2::IDENTITY),
            Velocity::new(Vec2::new(0.5, 0.0)),
            ChildOf(parent),
        ))
        .id();

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(1.5, 0.0), epsilon = 1e-4);

    app.world_mut()
        .entity_mut(parent)
        .insert(Velocity::new(Vec2::new(2.0, 1.0)).with_angular(0.5));

    run_sync_kinematics(&mut app);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(2.5, 2.0), epsilon = 1e-4);
    assert_relative_eq!(child_velocity.angular(), 0.5);
}

#[test]
fn sync_kinematics_velocity_complex_hierarchy() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let grandparent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.0, 0.0), Rot2::radians(FRAC_PI_4)),
            Velocity::new(Vec2::new(2.0, 0.0)).with_angular(0.2),
            ChildOf(layer),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(3.0, 1.0), Rot2::radians(0.5236)),
            Velocity::new(Vec2::new(1.0, 1.0)).with_angular(0.15),
            ChildOf(grandparent),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, -1.0), Rot2::radians(-FRAC_PI_4)),
            Velocity::new(Vec2::new(0.5, 1.5)).with_angular(0.1),
            ChildOf(parent),
        ))
        .id();

    let grandparent_velocity = app.world().get::<GlobalVelocity>(grandparent).unwrap();
    assert_relative_eq!(
        grandparent_velocity.linear(),
        Vec2::new(2.0, 0.0),
        epsilon = 1e-4
    );
    assert_relative_eq!(grandparent_velocity.angular(), 0.2);

    let parent_velocity = app.world().get::<GlobalVelocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.434315, 1.697056),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.35);

    let child_velocity = app.world().get::<GlobalVelocity>(child).unwrap();
    assert_relative_eq!(
        child_velocity.linear(),
        Vec2::new(-0.4707278, 3.087493),
        epsilon = 1e-4
    );
    assert_relative_eq!(child_velocity.angular(), 0.45, epsilon = 1e-4);
}

#[test]
fn sync_kinematics_grandparent_parent_changed() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();

    let grandparent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(2.0, 3.0), Rot2::IDENTITY),
            ChildOf(layer),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(1.0, 0.5), Rot2::IDENTITY),
            ChildOf(grandparent),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Position::new(Vec2::new(0.5, 0.25), Rot2::IDENTITY),
            ChildOf(parent),
        ))
        .id();

    let grandparent_tile = app.world().get::<TilePosition>(grandparent).unwrap();
    assert_eq!(grandparent_tile.layer(), layer);
    assert_eq!(grandparent_tile.position(), IVec2::new(2, 3));

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(3, 3));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(3, 3));

    let child_layer_pos = app.world().get::<GlobalPosition>(child).unwrap();
    assert_relative_eq!(child_layer_pos.position(), Vec2::new(3.5, 3.75));

    app.world_mut()
        .entity_mut(grandparent)
        .insert(Position::new(Vec2::new(5.0, 1.0), Rot2::IDENTITY));

    run_sync_kinematics(&mut app);

    let grandparent_tile = app.world().get::<TilePosition>(grandparent).unwrap();
    assert_eq!(grandparent_tile.layer(), layer);
    assert_eq!(grandparent_tile.position(), IVec2::new(5, 1));

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(6, 1));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(6, 1));

    let child_layer_pos = app.world().get::<GlobalPosition>(child).unwrap();
    assert_relative_eq!(child_layer_pos.position(), Vec2::new(6.5, 1.75));

    let index = app.world().resource::<TileIndex>();
    let grandparent_entities = index.get(TilePosition::new(layer, 5, 1));
    assert_eq!(grandparent_entities, &[grandparent]);

    let entities_at_6_1 = index.get(TilePosition::new(layer, 6, 1));
    assert!(entities_at_6_1.contains(&parent));
    assert!(entities_at_6_1.contains(&child));
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        TimePlugin,
        TilePlugin,
        KinematicsPlugin,
    ));
    app
}

fn run_sync_kinematics(app: &mut App) {
    app.world_mut().run_system_once(sync_kinematics).unwrap();
}
