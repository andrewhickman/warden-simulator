use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI},
    time::Duration,
};

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::{TimePlugin, TimeUpdateStrategy, prelude::*};
use bevy_transform::prelude::*;

use crate::{
    kinematics::{Position, RelativeVelocity, Velocity},
    layer::Layer,
    sync::{SyncPlugin, quat_to_rot},
    tile::{TilePlugin, TilePosition, index::TileIndex},
};

#[test]
fn quat_to_rot2_identity() {
    let rot = quat_to_rot(Quat::IDENTITY);
    assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
}

#[test]
fn quat_to_rot2_z() {
    for angle in [
        -0.7853982, 0.0, 0.7853982, 1.0, 1.570796, 3.141593, 4.712389, 6.283185, 10.0,
    ] {
        let rot = quat_to_rot(Quat::from_rotation_z(angle));
        let expected = Rot2::radians(angle);
        assert_relative_eq!(rot, expected, epsilon = 1e-4);
    }
}

#[test]
fn quat_to_rot2_x() {
    let rot = quat_to_rot(Quat::from_rotation_x(PI / 2.0));
    assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
}

#[test]
fn quat_to_rot2_y() {
    let rot = quat_to_rot(Quat::from_rotation_y(PI / 2.0));
    assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
}

#[test]
fn quat_to_rot2_xy() {
    let quat = Quat::from_rotation_x(1.0) * Quat::from_rotation_y(-1.5);
    let rot = quat_to_rot(quat);
    assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
}

#[test]
fn quat_to_rot2_xz() {
    let angle = PI / 3.0;
    let rot = quat_to_rot(Quat::from_rotation_x(PI / 4.0) * Quat::from_rotation_z(angle));
    assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
}

#[test]
fn quat_to_rot2_yz() {
    let angle = PI / 3.0;
    let rot = quat_to_rot(Quat::from_rotation_y(PI / 3.0) * Quat::from_rotation_z(angle));
    assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
}

#[test]
fn quat_to_rot2_xyz() {
    let angle = 1.234;
    let quat = Quat::from_euler(EulerRot::XYZ, 1.5, 0.4, angle);
    let rot = quat_to_rot(quat);
    assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
}

#[test]
fn sync_kinematics_transform_added() {
    let mut app = make_app();

    let layer = app.world_mut().spawn(Layer::default()).id();
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, -0.3, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4)),
            ChildOf(layer),
            Position::default(),
        ))
        .id();

    app.update();

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));

    let layer_pos = app.world().get::<Position>(entity).unwrap();
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
            Transform::from_xyz(1.2, -0.3, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4)),
            ChildOf(layer),
            Position::default(),
        ))
        .id();

    app.update();

    app.world_mut().entity_mut(entity).insert(
        Transform::from_xyz(2.1, -0.2, 0.0).with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
    );

    app.update();

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(2, -1));

    let layer_pos = app.world().get::<Position>(entity).unwrap();
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
            Transform::from_xyz(2.3, 1.7, 0.0),
            ChildOf(layer1),
            TilePosition::new(layer1, 2, 1),
            Position::default(),
        ))
        .id();

    app.update();

    app.world_mut().entity_mut(entity).insert(ChildOf(layer2));

    app.update();

    let tile = app.world().get::<TilePosition>(entity).unwrap();
    assert_eq!(tile.layer(), layer2);
    assert_eq!(tile.position(), IVec2::new(2, 1));

    let layer_pos = app.world().get::<Position>(entity).unwrap();
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
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            Position::default(),
        ))
        .id();

    app.update();

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

    app.update();

    let tile = app
        .world()
        .entity(entity)
        .get_ref::<TilePosition>()
        .unwrap();
    assert_eq!(tile.layer(), layer);
    assert_eq!(tile.position(), IVec2::new(1, -1));
    assert_eq!(tile.last_changed(), tile_change_tick);

    let layer_pos = app.world().get::<Position>(entity).unwrap();
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
            Transform::from_xyz(2.0, 3.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4)),
            ChildOf(layer),
            Position::default(),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.5, 0.5, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4)),
            ChildOf(parent),
            Position::default(),
        ))
        .id();

    app.update();

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(2, 3));

    let parent_layer_pos = app.world().get::<Position>(parent).unwrap();
    assert_relative_eq!(parent_layer_pos.position(), Vec2::new(2.0, 3.0));
    assert_relative_eq!(parent_layer_pos.rotation().as_radians(), FRAC_PI_4);

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(2, 4));

    let child_layer_pos = app.world().get::<Position>(child).unwrap();
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
        .insert(Transform::from_xyz(4.0, 1.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)));

    app.update();

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(4, 1));

    let parent_layer_pos = app.world().get::<Position>(parent).unwrap();
    assert_relative_eq!(parent_layer_pos.position(), Vec2::new(4.0, 1.0));
    assert_relative_eq!(parent_layer_pos.rotation().as_radians(), FRAC_PI_2);

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(3, 2));

    let child_layer_pos = app.world().get::<Position>(child).unwrap();
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
            Transform::from_xyz(1.2, -0.3, 0.0),
            ChildOf(layer),
            TilePosition::new(layer, 1, -1),
            Position::default(),
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
            Transform::from_xyz(1.0, 2.0, 0.0),
            RelativeVelocity::new(Vec2::new(3.0, 4.0)).with_angular(0.5),
            ChildOf(layer),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(2.0, 1.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(0.5, 0.5)),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::new(2.0, 1.0));
    assert_relative_eq!(parent_velocity.angular(), 0.0);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::ZERO).with_angular(1.0),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::ZERO),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::ZERO);
    assert_relative_eq!(parent_velocity.angular(), 1.0);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(3.0, 0.0)).with_angular(0.5),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 4.0, 0.0),
            RelativeVelocity::new(Vec2::new(1.0, 1.0)).with_angular(0.2),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(parent_velocity.linear(), Vec2::new(3.0, 0.0));
    assert_relative_eq!(parent_velocity.angular(), 0.5);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            RelativeVelocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.0, 0.0),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.0);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(1.0, 0.0)).with_angular(0.1),
            ChildOf(layer),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(0.0, 1.0)).with_angular(0.2),
            ChildOf(grandparent),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 3.0, 0.0),
            RelativeVelocity::new(Vec2::new(0.5, 0.5)).with_angular(0.3),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let grandparent_velocity = app.world().get::<Velocity>(grandparent).unwrap();
    assert_relative_eq!(grandparent_velocity.linear(), Vec2::new(1.0, 0.0));
    assert_relative_eq!(grandparent_velocity.angular(), 0.1);

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.0, 1.2),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.3);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(0.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(1.0, 0.0)),
            ChildOf(layer),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            RelativeVelocity::new(Vec2::new(0.5, 0.0)),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
    assert_relative_eq!(child_velocity.linear(), Vec2::new(1.5, 0.0), epsilon = 1e-4);

    app.world_mut()
        .entity_mut(parent)
        .insert(RelativeVelocity::new(Vec2::new(2.0, 1.0)).with_angular(0.5));

    app.update();

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(1.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4)),
            RelativeVelocity::new(Vec2::new(2.0, 0.0)).with_angular(0.2),
            ChildOf(layer),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(3.0, 1.0, 0.0).with_rotation(Quat::from_rotation_z(0.5236)),
            RelativeVelocity::new(Vec2::new(1.0, 1.0)).with_angular(0.15),
            ChildOf(grandparent),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, -1.0, 0.0).with_rotation(Quat::from_rotation_z(-FRAC_PI_4)),
            RelativeVelocity::new(Vec2::new(0.5, 1.5)).with_angular(0.1),
            ChildOf(parent),
        ))
        .id();

    app.update();

    let grandparent_velocity = app.world().get::<Velocity>(grandparent).unwrap();
    assert_relative_eq!(
        grandparent_velocity.linear(),
        Vec2::new(2.0, 0.0),
        epsilon = 1e-4
    );
    assert_relative_eq!(grandparent_velocity.angular(), 0.2);

    let parent_velocity = app.world().get::<Velocity>(parent).unwrap();
    assert_relative_eq!(
        parent_velocity.linear(),
        Vec2::new(1.434315, 1.697056),
        epsilon = 1e-4
    );
    assert_relative_eq!(parent_velocity.angular(), 0.35);

    let child_velocity = app.world().get::<Velocity>(child).unwrap();
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
            Transform::from_xyz(2.0, 3.0, 0.0),
            ChildOf(layer),
            Position::default(),
        ))
        .id();

    let parent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.0, 0.5, 0.0),
            ChildOf(grandparent),
            Position::default(),
        ))
        .id();

    let child = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.25, 0.0),
            ChildOf(parent),
            Position::default(),
        ))
        .id();

    app.update();

    let grandparent_tile = app.world().get::<TilePosition>(grandparent).unwrap();
    assert_eq!(grandparent_tile.layer(), layer);
    assert_eq!(grandparent_tile.position(), IVec2::new(2, 3));

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(3, 3));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(3, 3));

    let child_layer_pos = app.world().get::<Position>(child).unwrap();
    assert_relative_eq!(child_layer_pos.position(), Vec2::new(3.5, 3.75));

    app.world_mut()
        .entity_mut(grandparent)
        .insert(Transform::from_xyz(5.0, 1.0, 0.0));

    app.update();

    let grandparent_tile = app.world().get::<TilePosition>(grandparent).unwrap();
    assert_eq!(grandparent_tile.layer(), layer);
    assert_eq!(grandparent_tile.position(), IVec2::new(5, 1));

    let parent_tile = app.world().get::<TilePosition>(parent).unwrap();
    assert_eq!(parent_tile.layer(), layer);
    assert_eq!(parent_tile.position(), IVec2::new(6, 1));

    let child_tile = app.world().get::<TilePosition>(child).unwrap();
    assert_eq!(child_tile.layer(), layer);
    assert_eq!(child_tile.position(), IVec2::new(6, 1));

    let child_layer_pos = app.world().get::<Position>(child).unwrap();
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
        SyncPlugin,
    ));

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
    app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_duration(Duration::ZERO);

    app
}
