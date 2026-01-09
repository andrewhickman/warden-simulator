use std::time::Duration;

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::{TimePlugin, TimeUpdateStrategy, prelude::*};
use bevy_transform::prelude::*;

use crate::{
    collision::{Collision, CollisionTarget, Collisions},
    kinematics::{KinematicsPlugin, Velocity},
    tile::TilePlugin,
};

#[test]
fn update_kinematics() {
    let mut app = make_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(5.0, 3.0)),
        ))
        .id();

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(5.0, 3.0));
}

#[test]
fn update_kinematics_zero_velocity() {
    let mut app = make_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(10.0, 20.0, 0.0),
            Velocity::new(Vec2::ZERO),
        ))
        .id();

    let change_tick = app
        .world()
        .entity(entity)
        .get_ref::<Transform>()
        .unwrap()
        .last_changed();

    app.update();

    let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(10.0, 20.0));
    assert_eq!(transform.last_changed(), change_tick);
}

#[test]
fn update_kinematics_angular_velocity() {
    let mut app = make_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::ZERO).with_angular(1.0),
        ))
        .id();

    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    let expected = Quat::from_rotation_z(1.0);
    assert_relative_eq!(transform.rotation.x, expected.x, epsilon = 1e-6);
    assert_relative_eq!(transform.rotation.y, expected.y, epsilon = 1e-6);
    assert_relative_eq!(transform.rotation.z, expected.z, epsilon = 1e-6);
    assert_relative_eq!(transform.rotation.w, expected.w, epsilon = 1e-6);
}

#[test]
fn update_kinematics_wall_collision_active() {
    let mut app = make_app();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Default::default(),
            normal: Dir2::X,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: true,
        },
        0.0,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(-5.0, 3.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(0.0, 3.0));

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(0.0, 3.0));
}

#[test]
fn update_kinematics_wall_collision() {
    let mut app = make_app();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Vec2::new(2.0, -1.0),
            normal: Dir2::Y,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: true,
        },
        0.5,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(4.0, -2.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(4.0, 0.0));

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(4.0, -1.0));
}

#[test]
fn update_kinematics_multiple_collisions() {
    let mut app = make_app();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Default::default(),
            normal: Dir2::X,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: true,
        },
        0.0,
    );
    collisions.insert(
        Collision {
            position: Default::default(),
            normal: Dir2::Y,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: true,
        },
        0.0,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(-3.0, -4.0)),
            collisions,
        ))
        .id();

    let change_tick = app
        .world()
        .entity(entity)
        .get_ref::<Transform>()
        .unwrap()
        .last_changed();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::ZERO);

    let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::ZERO);
    assert_eq!(transform.last_changed(), change_tick);
}

#[test]
fn update_kinematics_wall_collision_receding() {
    let mut app = make_app();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Default::default(),
            normal: Dir2::X,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: true,
        },
        0.0,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(5.0, 3.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(5.0, 3.0));

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(5.0, 3.0));
}

#[test]
fn update_kinematics_collider_collision() {
    let mut app = make_app();

    let other_entity = app.world_mut().spawn_empty().id();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Vec2::new(-2.0, 1.0),
            normal: Dir2::new(Vec2::new(1.0, 1.0)).unwrap(),
            target: CollisionTarget::Collider {
                id: other_entity,
                position: Vec2::new(5.0, 5.0),
            },
            solid: true,
        },
        0.5,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(-4.0, -2.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(-1.0, 1.0), epsilon = 1e-4);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(
        transform.translation.xy(),
        Vec2::new(-2.5, 1.5),
        epsilon = 1e-4
    );
}

#[test]
fn update_kinematics_non_solid_collision_active() {
    let mut app = make_app();

    let other_entity = app.world_mut().spawn_empty().id();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Vec2::new(2.0, 1.0),
            normal: Dir2::X,
            target: CollisionTarget::Collider {
                id: other_entity,
                position: Vec2::new(3.0, 1.0),
            },
            solid: false,
        },
        0.5,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(4.0, 2.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(4.0, 2.0));

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(4.0, 2.0));
}

#[test]
fn update_kinematics_non_solid_wall_collision_active() {
    let mut app = make_app();

    let mut collisions = Collisions::default();
    collisions.insert(
        Collision {
            position: Vec2::new(2.0, 1.0),
            normal: Dir2::Y,
            target: CollisionTarget::Tile {
                id: None,
                position: Default::default(),
            },
            solid: false,
        },
        0.0,
    );

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Velocity::new(Vec2::new(3.0, -5.0)),
            collisions,
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity>(entity).unwrap();
    assert_relative_eq!(velocity.linear(), Vec2::new(3.0, -5.0));

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.xy(), Vec2::new(3.0, -5.0));
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        TimePlugin,
        TilePlugin,
        KinematicsPlugin,
    ));

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
    app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_duration(Duration::ZERO);

    app
}
