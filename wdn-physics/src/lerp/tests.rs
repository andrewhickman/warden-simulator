use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4, PI},
    time::Duration,
};

use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::{TimePlugin, prelude::*};

use super::*;

#[test]
fn position_spawned_fixed_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.5, -2.0), Rot2::radians(0.5)),
        ))
        .id();

    run_interpolate(&mut app, 1.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.5, -2.0, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(0.5));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.5, -2.0));
        }
        _ => panic!("expected Static translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(0.5));
        }
        _ => panic!("expected Static rotation, got {state:?}"),
    }
}

#[test]
fn position_spawned_render_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.5, -2.0), Rot2::radians(0.5)),
        ))
        .id();

    run_interpolate(&mut app, 0.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.5, -2.0, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(0.5));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.5, -2.0));
        }
        _ => panic!("expected Static translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(0.5));
        }
        _ => panic!("expected Static rotation, got {state:?}"),
    }
}

#[test]
fn fixed_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
        ))
        .id();

    run_interpolate(&mut app, 0.5);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );

    run_interpolate(&mut app, 1.0);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(0.5, 0.5, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_4));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::ZERO);
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!("expected Interpolating translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::IDENTITY);
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!("expected Interpolating rotation, got {state:?}"),
    }
}

#[test]
fn consecutive_fixed_updates() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.5, -2.0), Rot2::radians(FRAC_PI_2)),
        ))
        .id();

    run_interpolate(&mut app, 1.0);

    set_position(&mut app, entity, Vec2::new(1.0, -1.0), Rot2::IDENTITY);

    run_interpolate(&mut app, 1.5);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.25, -1.5, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_4));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(1.5, -2.0));
            assert_relative_eq!(end, Vec2::new(1.0, -1.0));
        }
        _ => panic!("expected Interpolating translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::radians(FRAC_PI_2));
            assert_relative_eq!(end, Rot2::IDENTITY);
        }
        _ => panic!("expected Interpolating rotation, got {state:?}"),
    }
}

#[test]
fn render_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(0.5, 0.5), Rot2::radians(FRAC_PI_2)),
        ))
        .id();

    run_interpolate(&mut app, 1.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.5, 1.5),
        Rot2::radians(-FRAC_PI_2),
    );

    run_interpolate(&mut app, 0.5);

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.5, 1.5, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(-FRAC_PI_2));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.5, 1.5));
        }
        _ => panic!("expected Static translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(-FRAC_PI_2));
        }
        _ => panic!("expected Static rotation, got {state:?}"),
    }
}

#[test]
fn consecutive_render_updates() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::ZERO, Rot2::IDENTITY),
        ))
        .id();

    run_interpolate(&mut app, 1.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );

    run_interpolate(&mut app, 1.3);
    run_interpolate(&mut app, 0.4);

    let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(0.7, 0.7, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(0.7 * FRAC_PI_2));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::ZERO);
            assert_relative_eq!(end, Vec2::new(1.0, 1.0));
        }
        _ => panic!("expected Interpolating translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::IDENTITY);
            assert_relative_eq!(end, Rot2::radians(FRAC_PI_2));
        }
        _ => panic!("expected Interpolating rotation, got {state:?}"),
    }
}

#[test]
fn position_modified_fixed_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::ZERO, Rot2::radians(FRAC_PI_4)),
        ))
        .id();

    run_interpolate(&mut app, 0.5);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );
    run_interpolate(&mut app, 1.0);

    set_position(&mut app, entity, Vec2::new(2.0, 2.0), Rot2::radians(PI));

    run_interpolate(&mut app, 1.0);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.5, 1.5, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(3.0 * FRAC_PI_4));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Vec2::new(1.0, 1.0));
            assert_relative_eq!(end, Vec2::new(2.0, 2.0));
        }
        _ => panic!("expected Interpolating translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Interpolating { start, end } => {
            assert_relative_eq!(start, Rot2::radians(FRAC_PI_2));
            assert_relative_eq!(end, Rot2::radians(PI));
        }
        _ => panic!("expected Interpolating rotation, got {state:?}"),
    }
}

#[test]
fn position_modified_render_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::ZERO, Rot2::radians(FRAC_PI_4)),
        ))
        .id();

    run_interpolate(&mut app, 0.0);

    set_position(
        &mut app,
        entity,
        Vec2::new(1.0, 1.0),
        Rot2::radians(FRAC_PI_2),
    );

    run_interpolate(&mut app, 1.3);

    set_position(
        &mut app,
        entity,
        Vec2::new(2.0, 2.0),
        Rot2::radians(FRAC_PI_4),
    );

    run_interpolate(&mut app, 0.4);

    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(2.0, 2.0, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_4));
    assert_relative_eq!(transform.scale, Vec3::ONE);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(2.0, 2.0));
        }
        _ => panic!("expected Static translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(FRAC_PI_4));
        }
        _ => panic!("expected Static rotation, got {state:?}"),
    }
}

#[test]
fn position_not_modified_render_update() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Interpolate::default(),
            Position::new(Vec2::new(1.0, 1.0), Rot2::radians(FRAC_PI_4)),
        ))
        .id();

    run_interpolate(&mut app, 0.0);

    let initial_transform_tick = app
        .world()
        .entity(entity)
        .get_change_ticks::<Transform>()
        .unwrap()
        .changed;

    run_interpolate(&mut app, 1.3);

    let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
    assert_relative_eq!(transform.translation, Vec3::new(1.0, 1.0, 0.0));
    assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_4));
    assert_relative_eq!(transform.scale, Vec3::ONE);
    assert_eq!(transform.last_changed(), initial_transform_tick);

    let state = app.world().get::<InterpolateState>(entity).unwrap();
    match state.translation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Vec2::new(1.0, 1.0));
        }
        _ => panic!("expected Static translation, got {state:?}"),
    }

    match state.rotation {
        ComponentInterpolateState::Static { value } => {
            assert_relative_eq!(value, Rot2::radians(FRAC_PI_4));
        }
        _ => panic!("expected Static rotation, got {state:?}"),
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

fn run_interpolate(app: &mut App, overstep: f32) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(Duration::from_secs_f32(overstep));
    app.update();
}
