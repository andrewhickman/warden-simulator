use std::{cmp::Reverse, f32::consts::FRAC_1_SQRT_2, time::Duration};

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::{FloatOrd, prelude::*};
use bevy_time::{TimePlugin, TimeUpdateStrategy, prelude::*};
use bevy_transform::prelude::*;

use crate::{
    collision::{
        Collider, ColliderDisabled, CollisionPlugin, CollisionTarget, Collisions, TileCollider,
    },
    integrate::Velocity,
    tile::{
        TilePlugin, TilePosition,
        storage::{TileLayer, TileMaterial, TileStorageMut},
    },
};

#[test]
fn collision_empty() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        0.5,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next().is_none());
    assert!(collisions.next_time().is_none());
}

#[test]
fn collision_collider() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(-0.5, 0.2),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(-0.1, 0.1),
        Vec2::new(0.5, 0.2),
        0.05,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.4);
    let collision1 = collisions1.next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.18));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.1, 0.18));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert_relative_eq!(collisions2.next_time().unwrap(), 0.4);
    let collision2 = collisions2.next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.1, 0.18));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.2, 0.18));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(0.5, 0.2),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(-0.1, 0.1),
        Vec2::new(-0.5, 0.2),
        0.05,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
}

#[test]
fn collision_collider_touching() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(0.0, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
}

#[test]
fn collision_collider_touching_and_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(0.5, 0.2),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(-0.5, 0.2),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
}

#[test]
fn collision_collider_touching_and_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(-0.5, 0.2),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(0.5, 0.2),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());
    let collision1 = collisions1.active().next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.4, 0.1));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.2, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 1);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
    let collision2 = collisions2.active().next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.2, 0.1));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.4, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(0.0, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.15, 0.1),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());
    let collision1 = collisions1.active().next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.15, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 1);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
    let collision2 = collisions2.active().next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.2, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_intersecting_and_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(0.5, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.15, 0.1),
        Vec2::new(-0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());
    let collision1 = collisions1.active().next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.15, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 1);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
    let collision2 = collisions2.active().next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.2, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_intersecting_and_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.1),
        Vec2::new(-0.5, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.15, 0.1),
        Vec2::new(0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    assert!(collisions1.next_time().is_none());
    assert!(collisions1.next().is_none());
    let collision1 = collisions1.active().next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.1));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.15, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 1);
    assert!(collisions2.next_time().is_none());
    assert!(collisions2.next().is_none());
    let collision2 = collisions2.active().next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.15, 0.1));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.2, 0.1));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_angled() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.3),
        Vec2::new(-0.5, 0.0),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.1, 0.22),
        Vec2::new(0.5, 0.0),
        0.05,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.24, epsilon = 0.0001);
    let collision1 = collisions1.next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.28, 0.3), epsilon = 0.0001);
    assert_relative_eq!(*collision1.normal, Vec2::new(0.6, 0.8), epsilon = 0.0001);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(0.22, 0.22), epsilon = 0.0001);
            assert_relative_eq!(
                collision1.position.distance(position),
                0.1,
                epsilon = 0.0001,
            );
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert_relative_eq!(collisions2.next_time().unwrap(), 0.24, epsilon = 0.0001);
    let collision2 = collisions2.next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(0.22, 0.22), epsilon = 0.0001);
    assert_relative_eq!(*collision2.normal, Vec2::new(-0.6, -0.8), epsilon = 0.0001);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.28, 0.3), epsilon = 0.0001);
            assert_relative_eq!(
                collision2.position.distance(position),
                0.1,
                epsilon = 0.0001,
            );
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_almost_touching_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let t = 1e-6f32;
    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2 + t, 0.0),
        Vec2::new(-0.5, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), t);
    let collision1 = collisions1.next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2 + t / 2.0, 0.0));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(t / 2.0, 0.0));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert_relative_eq!(collisions2.next_time().unwrap(), t);
    let collision2 = collisions2.next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(t / 2.0, 0.0));
    assert_eq!(collision2.normal, Dir2::NEG_X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.2 + t / 2.0, 0.0));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_wall_north_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.8),
        Vec2::new(0.0, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
}

#[test]
fn collision_wall_north_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.8),
        Vec2::new(0.0, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
    assert_eq!(collision.normal, Dir2::NEG_Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_north_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.9),
        Vec2::new(0.0, 0.0),
        0.2,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
    assert_eq!(collision.normal, Dir2::NEG_Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_south_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.2),
        Vec2::new(0.0, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
}

#[test]
fn collision_wall_south_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.2),
        Vec2::new(0.0, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.1));
    assert_eq!(collision.normal, Dir2::Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_south_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.1),
        Vec2::new(0.0, 0.0),
        0.2,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.1));
    assert_eq!(collision.normal, Dir2::Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_east_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.8, 0.0),
        Vec2::new(-0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
}

#[test]
fn collision_wall_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.8, 0.0),
        Vec2::new(0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.9, 0.0));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 0));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_east_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.9, 0.0),
        Vec2::new(0.0, 0.0),
        0.2,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.9, 0.0));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 0));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_west_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.0),
        Vec2::new(0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
}

#[test]
fn collision_wall_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.0),
        Vec2::new(-0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.1, 0.0));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 0));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_west_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.1, 0.0),
        Vec2::new(0.0, 0.0),
        0.2,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.1, 0.0));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 0));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_north_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.8),
        Vec2::new(0.0, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
    assert_eq!(collision.normal, Dir2::NEG_Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, 1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_south_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 0, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.2),
        Vec2::new(0.0, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.1));
    assert_eq!(collision.normal, Dir2::Y);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, -1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.8, 0.0),
        Vec2::new(0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.9, 0.0));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 0));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, -1, 0));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.0),
        Vec2::new(-0.5, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.2);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.1, 0.0));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 0));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_corner_north_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.7),
        Vec2::new(0.5, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.92928934, 0.92928934));
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_corner_north_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, -1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.3, 0.7),
        Vec2::new(-0.5, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.07071069, 0.92928934),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_corner_south_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, -1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.3, 0.3),
        Vec2::new(-0.5, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.07071069, 0.07071069),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, -1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_corner_south_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.3),
        Vec2::new(0.5, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.92928934, 0.07071069),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, -1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_north_east_receding() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.85, 0.85),
        Vec2::new(-0.5, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
}

#[test]
fn collision_corner_north_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.7),
        Vec2::new(0.5, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.92928934, 0.92928934));
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_north_east_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.95, 0.95),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.95, 0.95), epsilon = 0.0001);
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_north_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.3, 0.7),
        Vec2::new(-0.5, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.07071069, 0.92928934),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_north_west_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.05, 0.95),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.05, 0.95), epsilon = 0.0001);
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_south_west_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.3, 0.3),
        Vec2::new(-0.5, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.07071069, 0.07071069),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_south_west_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, -1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.05, 0.05),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.05, 0.05), epsilon = 0.0001);
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, -1, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_south_east_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.3),
        Vec2::new(0.5, -0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_eq!(collisions.next_time().unwrap(), 0.45857865);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(0.92928934, 0.07071069),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_south_east_intersecting() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, -1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.95, 0.05),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.95, 0.05), epsilon = 0.0001);
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, -1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_angled() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.31, 0.6),
        Vec2::new(1.0, 0.5),
        0.05,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.7040017, epsilon = 0.0001);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(
        collision.position,
        Vec2::new(1.0140017, 0.95200086),
        epsilon = 0.0001
    );
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(0.28003645, -0.95998937),
        epsilon = 0.0001
    );
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_wall_ordering() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 0));
    set_tile(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.7),
        Vec2::new(0.5, 0.3),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 0);
    assert_relative_eq!(collisions.next_time().unwrap(), 0.4);
    let collision = collisions.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.9, 0.82));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 0));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_intersecting_and_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.3, 0.5),
        Vec2::new(1.0, 0.0),
        0.1,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.32, 0.5),
        Vec2::new(0.0, 0.0),
        0.1,
    );
    let entity3 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.8, 0.5),
        Vec2::new(0.0, 0.0),
        0.1,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    let active_collision = collisions1.active().next().unwrap();
    assert_relative_eq!(active_collision.position, Vec2::new(0.3, 0.5));
    assert_eq!(active_collision.normal, Dir2::NEG_X);
    assert!(active_collision.solid);
    match active_collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }

    assert_relative_eq!(collisions1.next_time().unwrap(), 0.3);
    let next_collision = collisions1.next().unwrap();
    assert_relative_eq!(next_collision.position, Vec2::new(0.6, 0.5));
    assert_eq!(next_collision.normal, Dir2::NEG_X);
    assert!(next_collision.solid);
    match next_collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity3);
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_ordering() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.5),
        Vec2::new(0.5, 0.0),
        0.01,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.32, 0.5),
        Vec2::new(0.0, 0.0),
        0.01,
    );
    let entity3 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.44, 0.5),
        Vec2::new(-0.5, 0.0),
        0.01,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.2);
    let collision = collisions1.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.3, 0.5));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions3 = app.world().get::<Collisions>(entity3).unwrap();
    assert_eq!(collisions3.active().len(), 0);
    assert_relative_eq!(collisions3.next_time().unwrap(), 0.2);
    let collision = collisions3.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.34, 0.5));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_colliders_tile_boundary() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.85, 0.5),
        Vec2::new(0.5, 0.0),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(1.15, 0.5),
        Vec2::new(-0.5, 0.0),
        0.05,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.2);
    let collision1 = collisions1.next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.95, 0.5));
    assert_eq!(collision1.normal, Dir2::NEG_X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity2);
            assert_relative_eq!(position, Vec2::new(1.05, 0.5));
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert_relative_eq!(collisions2.next_time().unwrap(), 0.2);
    let collision2 = collisions2.next().unwrap();
    assert_relative_eq!(collision2.position, Vec2::new(1.05, 0.5));
    assert_eq!(collision2.normal, Dir2::X);
    assert!(collision2.solid);
    match collision2.target {
        CollisionTarget::Collider { id, position } => {
            assert_eq!(id, entity1);
            assert_relative_eq!(position, Vec2::new(0.95, 0.5));
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_disabled_inserted() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(-0.5, 0.2),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(-0.1, 0.1),
        Vec2::new(0.5, 0.2),
        0.05,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert!(collisions1.next().is_some());

    app.world_mut().entity_mut(entity1).insert(ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next().is_none());

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next().is_none());
}

#[test]
fn collision_collider_disabled_removed() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(-0.5, 0.2),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(-0.1, 0.1),
        Vec2::new(0.5, 0.2),
        0.05,
    );

    app.world_mut().entity_mut(entity1).insert(ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert!(collisions1.next().is_none());

    app.world_mut()
        .entity_mut(entity1)
        .remove::<ColliderDisabled>();
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert!(collisions1.next().is_some());

    let collision1 = collisions1.next().unwrap();
    assert_relative_eq!(collision1.position, Vec2::new(0.2, 0.18));
    assert_eq!(collision1.normal, Dir2::X);
    assert!(collision1.solid);
    match collision1.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_disabled() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.4, 0.1),
        Vec2::new(-0.5, 0.2),
        0.05,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(-0.1, 0.1),
        Vec2::new(0.5, 0.2),
        0.05,
    );

    app.world_mut()
        .entity_mut(entity2)
        .insert(crate::collision::ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next().is_none());

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next().is_none());
}

#[test]
fn collision_tile_collider_disabled() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.8, 0.5),
        Vec2::new(1.0, 0.5),
        0.05,
    );

    let tile_position = TilePosition::new(layer, 1, 0);
    let tile_entity = spawn_tile_collider(&mut app, tile_position);

    app.world_mut()
        .entity_mut(tile_entity)
        .insert(ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next().is_none());
}

#[test]
fn collision_wall_disabled() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_position = TilePosition::new(layer, 0, 1);
    set_tile(&mut app, tile_position);

    let entity = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.5, 0.8),
        Vec2::new(0.0, 0.5),
        0.05,
    );

    app.world_mut().entity_mut(entity).insert(ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert!(collisions1.next().is_none());
}

#[test]
fn collision_collider_disabled_ordering() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.5),
        Vec2::new(0.5, 0.0),
        0.01,
    );
    let entity2 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.32, 0.5),
        Vec2::new(0.0, 0.0),
        0.01,
    );
    let entity3 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.44, 0.5),
        Vec2::new(-0.5, 0.0),
        0.01,
    );

    app.world_mut().entity_mut(entity2).insert(ColliderDisabled);
    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 0);
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.22);
    let collision = collisions1.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.31, 0.5));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity3);
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 0);
    assert!(collisions2.next().is_none());

    let collisions3 = app.world().get::<Collisions>(entity3).unwrap();
    assert_eq!(collisions3.active().len(), 0);
    assert_relative_eq!(collisions3.next_time().unwrap(), 0.22);
    let collision = collisions3.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.33, 0.5));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity1);
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_collider_non_solid_ordering() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let entity1 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.2, 0.5),
        Vec2::new(0.5, 0.0),
        0.01,
    );
    let entity2 = spawn_non_solid_collider(
        &mut app,
        layer,
        Vec2::new(0.32, 0.5),
        Vec2::new(0.0, 0.0),
        0.01,
    );
    let entity3 = spawn_collider(
        &mut app,
        layer,
        Vec2::new(0.44, 0.5),
        Vec2::new(-0.5, 0.0),
        0.01,
    );

    app.update();

    let collisions1 = app.world().get::<Collisions>(entity1).unwrap();
    assert_eq!(collisions1.active().len(), 1);
    let active1 = collisions1.active().next().unwrap();
    assert_relative_eq!(active1.position, Vec2::new(0.3, 0.5));
    assert_eq!(active1.normal, Dir2::NEG_X);
    assert!(!active1.solid);
    match active1.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }
    assert_relative_eq!(collisions1.next_time().unwrap(), 0.22);
    let collision = collisions1.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.31, 0.5));
    assert_eq!(collision.normal, Dir2::NEG_X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity3);
        }
        _ => panic!("Expected collider collision"),
    }

    let collisions2 = app.world().get::<Collisions>(entity2).unwrap();
    assert_eq!(collisions2.active().len(), 2);
    let mut active2: Vec<_> = collisions2.active().collect();
    active2.sort_by_key(|c| Reverse(FloatOrd(c.normal.x)));

    assert_relative_eq!(active2[0].position, Vec2::new(0.32, 0.5));
    assert_eq!(active2[0].normal, Dir2::X);
    assert!(!active2[0].solid);
    match active2[0].target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity1);
        }
        _ => panic!("Expected collider collision"),
    }

    assert_relative_eq!(active2[1].position, Vec2::new(0.32, 0.5));
    assert_eq!(active2[1].normal, Dir2::NEG_X);
    assert!(!active2[1].solid);
    match active2[1].target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity3);
        }
        _ => panic!("Expected collider collision"),
    }
    assert!(collisions2.next().is_none());

    let collisions3 = app.world().get::<Collisions>(entity3).unwrap();
    assert_eq!(collisions3.active().len(), 1);
    let active3 = collisions3.active().next().unwrap();
    assert_relative_eq!(active3.position, Vec2::new(0.34, 0.5));
    assert_eq!(active3.normal, Dir2::X);
    assert!(!active3.solid);
    match active3.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity2);
        }
        _ => panic!("Expected collider collision"),
    }
    assert_relative_eq!(collisions3.next_time().unwrap(), 0.22);
    let collision = collisions3.next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.33, 0.5));
    assert_eq!(collision.normal, Dir2::X);
    assert!(collision.solid);
    match collision.target {
        CollisionTarget::Collider { id, .. } => {
            assert_eq!(id, entity1);
        }
        _ => panic!("Expected collider collision"),
    }
}

#[test]
fn collision_wall_non_solid_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_non_solid_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.8),
        Vec2::new(0.0, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
    assert_eq!(collision.normal, Dir2::NEG_Y);
    assert!(!collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_tile_collider_non_solid_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    let tile_entity = spawn_tile_collider(&mut app, TilePosition::new(layer, 0, 1));

    let entity = spawn_non_solid_collider(
        &mut app,
        layer,
        Vec2::new(0.0, 0.8),
        Vec2::new(0.0, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.0, 0.9));
    assert_eq!(collision.normal, Dir2::NEG_Y);
    assert!(!collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 0, 1));
            assert_eq!(id, Some(tile_entity));
        }
        _ => panic!("Expected wall collision"),
    }
}

#[test]
fn collision_corner_non_solid_closing() {
    let mut app = make_app();
    let layer = spawn_layer(&mut app);

    set_tile(&mut app, TilePosition::new(layer, 1, 1));

    let entity = spawn_non_solid_collider(
        &mut app,
        layer,
        Vec2::new(0.7, 0.7),
        Vec2::new(0.5, 0.5),
        0.1,
    );

    app.update();

    let collisions = app.world().get::<Collisions>(entity).unwrap();
    assert_eq!(collisions.active().len(), 1);
    assert!(collisions.next_time().is_none());
    assert!(collisions.next().is_none());
    let collision = collisions.active().next().unwrap();
    assert_relative_eq!(collision.position, Vec2::new(0.92928934, 0.92928934));
    assert_relative_eq!(
        *collision.normal,
        Vec2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        epsilon = 0.0001
    );
    assert!(!collision.solid);
    match collision.target {
        CollisionTarget::Wall { id, position } => {
            assert_eq!(position, TilePosition::new(layer, 1, 1));
            assert!(id.is_none());
        }
        _ => panic!("Expected wall collision"),
    }
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TimePlugin, TilePlugin, CollisionPlugin));

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
    app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));

    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_duration(Duration::ZERO);

    app
}

fn spawn_layer(app: &mut App) -> Entity {
    app.world_mut().spawn(TileLayer {}).id()
}

fn spawn_collider(
    app: &mut App,
    layer: Entity,
    position: Vec2,
    velocity: Vec2,
    radius: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Collider::new(radius, true),
            Transform::from_translation(position.extend(0.0)),
            Velocity::new(velocity),
            ChildOf(layer),
        ))
        .id()
}

fn spawn_non_solid_collider(
    app: &mut App,
    layer: Entity,
    position: Vec2,
    velocity: Vec2,
    radius: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Collider::new(radius, false),
            Transform::from_translation(position.extend(0.0)),
            Velocity::new(velocity),
            ChildOf(layer),
        ))
        .id()
}

fn spawn_tile_collider(app: &mut App, position: TilePosition) -> Entity {
    app.world_mut()
        .spawn((TileCollider, position, ChildOf(position.layer())))
        .id()
}

fn set_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(position, TileMaterial::Wall);
        })
        .unwrap();
}
