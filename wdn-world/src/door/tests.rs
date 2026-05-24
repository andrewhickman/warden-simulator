use std::time::Duration;

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_time::{TimePlugin, TimeUpdateStrategy, prelude::*};

use wdn_physics::collision::ColliderDisabled;

use super::{Door, DoorPlugin, DoorState};

#[test]
fn door_closed() {
    let mut app = make_app(Duration::from_secs(1));
    let id = app.world_mut().spawn(Door::default()).id();

    let initial_tick = app
        .world()
        .entity(id)
        .get_change_ticks::<Door>()
        .unwrap()
        .changed;

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Closed));
    assert_eq!(door.position(), 0.0);
    assert!(!app.world().entity(id).contains::<ColliderDisabled>());
    assert_eq!(
        app.world()
            .entity(id)
            .get_change_ticks::<Door>()
            .unwrap()
            .changed,
        initial_tick
    );
}

#[test]
fn door_open_autoclose() {
    let mut app = make_app(Duration::from_millis(400));
    let id = app.world_mut().spawn(Door::default()).id();

    app.world_mut().get_mut::<Door>(id).unwrap().open();

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Opening { .. }));
    assert_relative_eq!(door.position(), 0.4);
    assert!(!app.world().entity(id).contains::<ColliderDisabled>());

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Opening { .. }));
    assert_relative_eq!(door.position(), 0.8);
    assert!(app.world().entity(id).contains::<ColliderDisabled>());

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Open { .. }));
    assert_relative_eq!(door.position(), 1.0);
    assert!(app.world().entity(id).contains::<ColliderDisabled>());

    for _ in 0..7 {
        app.update();

        let door = app.world().entity(id).get_ref::<Door>().unwrap();
        assert!(matches!(door.state, DoorState::Open { .. }));
        assert_relative_eq!(door.position(), 1.0);
        assert!(app.world().entity(id).contains::<ColliderDisabled>());
    }

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Closing { .. }));
    assert_relative_eq!(door.position(), 1.0);
    assert!(app.world().entity(id).contains::<ColliderDisabled>());

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Closing { .. }));
    assert_relative_eq!(door.position(), 0.6);
    assert!(app.world().entity(id).contains::<ColliderDisabled>());

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Closing { .. }));
    assert_relative_eq!(door.position(), 0.2);
    assert!(!app.world().entity(id).contains::<ColliderDisabled>());

    app.update();

    let door = app.world().entity(id).get_ref::<Door>().unwrap();
    assert!(matches!(door.state, DoorState::Closed { .. }));
    assert_relative_eq!(door.position(), 0.0);
    assert!(!app.world().entity(id).contains::<ColliderDisabled>());
}

fn make_app(timestep: Duration) -> App {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TimePlugin, DoorPlugin));

    app.insert_resource(Time::<Fixed>::from_duration(timestep));
    app.insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(timestep));

    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_duration(Duration::ZERO);

    app
}
