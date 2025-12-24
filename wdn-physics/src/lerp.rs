use bevy::{
    ecs::{change_detection::Tick, system::SystemChangeTick},
    prelude::*,
};

pub struct InterpolatePlugin;

#[derive(Default, Debug, Clone, Copy, Component)]
#[require(InterpolateState)]
pub struct Interpolated;

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) enum InterpolateState {
    #[default]
    None,
    Fixed {
        start: Transform,
    },
    Interpolated {
        start: Transform,
        end: Transform,
        change_tick: Tick,
    },
}

fn end_interpolation(mut transforms: Query<(&mut Transform, &mut InterpolateState)>) {
    transforms
        .par_iter_mut()
        .for_each(|(mut transform, mut state)| {
            match *state {
                InterpolateState::Fixed { .. } => return,
                InterpolateState::Interpolated {
                    end, change_tick, ..
                } if transform.last_changed() == change_tick => {
                    *transform = end;
                    *state = InterpolateState::Fixed { start: end };
                }
                _ => *state = InterpolateState::Fixed { start: *transform },
            };
        });
}

fn start_interpolation(
    mut transforms: Query<(&mut Transform, &mut InterpolateState)>,
    time: Res<Time<Fixed>>,
    tick: SystemChangeTick,
) {
    let overstep = time.overstep_fraction();
    transforms
        .par_iter_mut()
        .for_each(|(mut transform, mut state)| {
            let (start, end) = match *state {
                InterpolateState::Fixed { start, .. } if start != *transform => (start, *transform),
                InterpolateState::Interpolated {
                    start,
                    end,
                    change_tick,
                } if transform.last_changed() == change_tick => (start, end),
                _ => {
                    *state = InterpolateState::None;
                    return;
                }
            };

            if start.translation != end.translation {
                transform.translation = start.translation.lerp(end.translation, overstep);
            }
            if start.rotation != end.rotation {
                transform.rotation = start.rotation.slerp(end.rotation, overstep);
            }
            if start.scale != end.scale {
                transform.scale = start.scale.lerp(end.scale, overstep);
            }

            *state = InterpolateState::Interpolated {
                start,
                end,
                change_tick: tick.this_run(),
            }
        });
}

impl Plugin for InterpolatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedFirst, end_interpolation);
        app.add_systems(
            RunFixedMainLoop,
            start_interpolation.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{
        f32::consts::{FRAC_PI_2, FRAC_PI_4, PI},
        time::Duration,
    };

    use approx::assert_relative_eq;
    use bevy::prelude::*;

    use super::*;

    #[test]
    fn transform_spawned_fixed_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(1.5, -2.0, 0.5),
                    rotation: Quat::from_rotation_y(0.5),
                    scale: Vec3::new(2.0, 1.5, 3.0),
                },
            ))
            .id();

        run_end_interpolation(&mut app);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.5, -2.0, 0.5));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_y(0.5));
        assert_relative_eq!(transform.scale, Vec3::new(2.0, 1.5, 3.0));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Fixed { start } => {
                assert_relative_eq!(start.translation, Vec3::new(1.5, -2.0, 0.5));
                assert_relative_eq!(start.rotation, Quat::from_rotation_y(0.5));
                assert_relative_eq!(start.scale, Vec3::new(2.0, 1.5, 3.0));
            }
            _ => panic!("expected Fixed interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn transform_spawned_render_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(1.5, -2.0, 0.5),
                    rotation: Quat::from_rotation_y(0.5),
                    scale: Vec3::new(2.0, 1.5, 3.0),
                },
            ))
            .id();

        run_start_interpolation(&mut app, 0.5);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.5, -2.0, 0.5));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_y(0.5));
        assert_relative_eq!(transform.scale, Vec3::new(2.0, 1.5, 3.0));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::None => {}
            _ => panic!("expected Fixed position, got {state:?}"),
        }
    }

    #[test]
    fn fixed_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((Interpolated, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, 1.0, 0.5),
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::splat(1.5),
            },
        );
        run_start_interpolation(&mut app, 0.5);

        run_end_interpolation(&mut app);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.0, 1.0, 0.5));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_x(FRAC_PI_2));
        assert_relative_eq!(transform.scale, Vec3::splat(1.5));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Fixed { start } => {
                assert_relative_eq!(start.translation, Vec3::new(1.0, 1.0, 0.5));
                assert_relative_eq!(start.rotation, Quat::from_rotation_x(FRAC_PI_2));
                assert_relative_eq!(start.scale, Vec3::splat(1.5));
            }
            _ => panic!("expected Fixed position, got {state:?}"),
        }
    }

    #[test]
    fn consecutive_fixed_updates() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(1.5, -2.0, 0.5),
                    rotation: Quat::from_rotation_z(FRAC_PI_4),
                    scale: Vec3::splat(2.0),
                },
            ))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, -1.0, 0.5),
                rotation: Quat::from_rotation_z(FRAC_PI_2),
                scale: Vec3::splat(1.5),
            },
        );
        run_end_interpolation(&mut app);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.0, -1.0, 0.5));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_z(FRAC_PI_2));
        assert_relative_eq!(transform.scale, Vec3::splat(1.5));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Fixed { start } => {
                assert_relative_eq!(start.translation, Vec3::new(1.5, -2.0, 0.5));
                assert_relative_eq!(start.rotation, Quat::from_rotation_z(FRAC_PI_4));
                assert_relative_eq!(start.scale, Vec3::splat(2.0));
            }
            _ => panic!("expected Fixed interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn render_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.5),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(2.0),
                },
            ))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, 1.0, 0.5),
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::splat(1.0),
            },
        );

        run_start_interpolation(&mut app, 0.5);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(0.5, 0.5, 0.5));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_x(FRAC_PI_4));
        assert_relative_eq!(transform.scale, Vec3::splat(1.5));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Interpolated {
                start,
                end,
                change_tick,
            } => {
                assert_relative_eq!(start.translation, Vec3::new(0.0, 0.0, 0.5));
                assert_relative_eq!(start.rotation, Quat::IDENTITY);
                assert_relative_eq!(start.scale, Vec3::splat(2.0));
                assert_relative_eq!(end.translation, Vec3::new(1.0, 1.0, 0.5));
                assert_relative_eq!(end.rotation, Quat::from_rotation_x(FRAC_PI_2));
                assert_relative_eq!(end.scale, Vec3::splat(1.0));
                assert_eq!(change_tick, transform.last_changed());
            }
            _ => panic!("expected Interpolated interpolation state, got {state:?}"),
        }

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(2.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(PI),
                scale: Vec3::splat(0.5),
            },
        );

        run_start_interpolation(&mut app, 0.0);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.5, 1.5, 0.25));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_x(3.0 * FRAC_PI_4));
        assert_relative_eq!(transform.scale, Vec3::splat(0.75));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Interpolated {
                start,
                end,
                change_tick,
            } => {
                assert_relative_eq!(start.translation, Vec3::new(1.0, 1.0, 0.5));
                assert_relative_eq!(start.rotation, Quat::from_rotation_x(FRAC_PI_2));
                assert_relative_eq!(start.scale, Vec3::splat(1.0));
                assert_relative_eq!(end.translation, Vec3::new(2.0, 2.0, 0.0));
                assert_relative_eq!(end.rotation, Quat::from_rotation_x(PI));
                assert_relative_eq!(end.scale, Vec3::splat(0.5));
                assert_eq!(change_tick, transform.last_changed());
            }
            _ => panic!("expected Interpolated interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn consecutive_render_updates() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            ))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, 1.0, 1.0),
                rotation: Quat::from_rotation_y(FRAC_PI_2),
                scale: Vec3::splat(2.0),
            },
        );

        run_start_interpolation(&mut app, 0.3);
        run_start_interpolation(&mut app, 0.4);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(0.7, 0.7, 0.7));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_y(0.7 * FRAC_PI_2));
        assert_relative_eq!(transform.scale, Vec3::splat(1.7));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Interpolated {
                start,
                end,
                change_tick,
            } => {
                assert_relative_eq!(start.translation, Vec3::new(0.0, 0.0, 0.0));
                assert_relative_eq!(start.rotation, Quat::IDENTITY);
                assert_relative_eq!(start.scale, Vec3::ONE);
                assert_relative_eq!(end.translation, Vec3::new(1.0, 1.0, 1.0));
                assert_relative_eq!(end.rotation, Quat::from_rotation_y(FRAC_PI_2));
                assert_relative_eq!(end.scale, Vec3::splat(2.0));
                assert_eq!(change_tick, transform.last_changed());
            }
            _ => panic!("expected Fixed interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn transform_modified_fixed_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Quat::from_rotation_z(FRAC_PI_4),
                    scale: Vec3::splat(1.5),
                },
            ))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, 1.0, 0.0),
                rotation: Quat::from_rotation_z(FRAC_PI_2),
                scale: Vec3::splat(2.0),
            },
        );
        run_start_interpolation(&mut app, 0.5);

        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(2.0, 2.0, 0.0),
                rotation: Quat::from_rotation_z(PI),
                scale: Vec3::splat(0.5),
            },
        );

        run_end_interpolation(&mut app);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(2.0, 2.0, 0.0));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_z(PI));
        assert_relative_eq!(transform.scale, Vec3::splat(0.5));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::Fixed { start } => {
                assert_relative_eq!(start.translation, Vec3::new(2.0, 2.0, 0.0));
                assert_relative_eq!(start.rotation, Quat::from_rotation_z(PI));
                assert_relative_eq!(start.scale, Vec3::splat(0.5));
            }
            _ => panic!("expected Fixed interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn transform_modified_render_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Quat::from_rotation_x(FRAC_PI_4),
                    scale: Vec3::splat(1.2),
                },
            ))
            .id();

        run_end_interpolation(&mut app);
        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(1.0, 1.0, 0.0),
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::splat(2.5),
            },
        );
        run_start_interpolation(&mut app, 0.3);

        set_transform(
            &mut app,
            entity,
            Transform {
                translation: Vec3::new(2.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(PI),
                scale: Vec3::splat(0.8),
            },
        );

        run_start_interpolation(&mut app, 0.4);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(2.0, 2.0, 0.0));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_x(PI));
        assert_relative_eq!(transform.scale, Vec3::splat(0.8));

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::None => {}
            _ => panic!("expected None interpolation state, got {state:?}"),
        }
    }

    #[test]
    fn transform_not_modified_render_update() {
        let mut app = make_app();
        let entity = app
            .world_mut()
            .spawn((
                Interpolated,
                Transform {
                    translation: Vec3::new(1.0, 1.0, 0.0),
                    rotation: Quat::from_rotation_y(FRAC_PI_4),
                    scale: Vec3::splat(1.8),
                },
            ))
            .id();

        let initial_transform_tick = app
            .world()
            .entity(entity)
            .get_change_ticks::<Transform>()
            .unwrap()
            .changed;

        run_end_interpolation(&mut app);
        run_start_interpolation(&mut app, 0.3);

        let transform = app.world().entity(entity).get_ref::<Transform>().unwrap();
        assert_relative_eq!(transform.translation, Vec3::new(1.0, 1.0, 0.0));
        assert_relative_eq!(transform.rotation, Quat::from_rotation_y(FRAC_PI_4));
        assert_relative_eq!(transform.scale, Vec3::splat(1.8));
        assert_eq!(transform.last_changed(), initial_transform_tick);

        let state = app.world().get::<InterpolateState>(entity).unwrap();
        match *state {
            InterpolateState::None => {}
            _ => panic!("expected None interpolation state, got {state:?}"),
        }
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InterpolatePlugin));
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));
        app
    }

    fn run_end_interpolation(app: &mut App) {
        app.world_mut().run_schedule(FixedFirst);
    }

    fn run_start_interpolation(app: &mut App, overstep: f32) {
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(Duration::from_secs_f32(overstep));
        app.world_mut().run_schedule(RunFixedMainLoop);
    }

    fn set_transform(app: &mut App, entity: Entity, transform: Transform) {
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap() = transform;
    }
}
