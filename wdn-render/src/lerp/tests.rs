use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4},
    time::Duration,
};

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::{TimePlugin, prelude::*};

use super::*;

#[test]
fn interpolate_added() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.0, 2.0), Rot2::radians(FRAC_PI_2)),
        ))
        .id();
    run_update(&mut app, 0.0);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(1.0, 2.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.0, 2.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_start() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(0.5, 0.5));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2 * 0.5));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(0.0, 0.0));
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::IDENTITY);
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_unchanged() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.0, 1.0), Rot2::radians(FRAC_PI_2)),
        ))
        .id();
    run_update(&mut app, 0.0);

    let initial_transform_tick = app
        .world()
        .entity(entity)
        .get_change_ticks::<Transform>()
        .unwrap()
        .changed;

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.5);

    let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(1.0, 1.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2));
    assert_eq!(transform.last_changed(), initial_transform_tick);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.0, 1.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_changed() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(0.5, 0.5),
        Rot2::radians(FRAC_PI_4),
    );
    run_update(&mut app, 1.25);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.25);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(0.75, 0.75));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2 * 0.75));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(0.5, 0.5));
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::radians(FRAC_PI_4));
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_continue() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.33);
    run_update(&mut app, 0.33);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(0.66, 0.66));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2 * 0.66));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(0.0, 0.0));
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::IDENTITY);
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_finish() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(0.5, 0.5),
        Rot2::radians(FRAC_PI_4),
    );
    run_update(&mut app, 1.25);

    set_position(
        &mut app,
        entity,
        Vec2::new(0.5, 0.5),
        Rot2::radians(FRAC_PI_4),
    );
    run_update(&mut app, 1.25);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(0.5, 0.5));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_4));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(0.5, 0.5));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(FRAC_PI_4));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Static",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_position() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::translation(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::new(0.5, 0.5));
    assert_relative_eq!(transform.rotation, Quat::IDENTITY);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(0.0, 0.0));
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Unset => {}
        _ => panic!(
            "expected interpolation state {:?} to be Unset",
            state.rotation
        ),
    }
}

#[test]
fn interpolate_rotation() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::rotation(),
            Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
        ))
        .id();
    run_update(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_update(&mut app, 1.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation.truncate(), Vec2::ZERO);
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2 * 0.5));

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Unset => {}
        _ => panic!(
            "expected interpolation state {:?} to be Unset",
            state.translation
        ),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::IDENTITY);
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!(
            "expected interpolation state {:?} to be Interpolating",
            state.rotation
        ),
    }
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TimePlugin, InterpolatePlugin));
    app.insert_resource(Time::<Fixed>::from_seconds(1.0));
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    app
}

fn set_position(app: &mut App, entity: Entity, position: Vec2, rotation: Rot2) {
    *app.world_mut().get_mut::<Position>(entity).unwrap() = Position::new(position, rotation);
}

fn run_update(app: &mut App, overstep: f32) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(Duration::from_secs_f32(overstep));
    app.update();
}
