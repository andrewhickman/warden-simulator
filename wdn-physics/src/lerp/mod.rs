#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::{change_detection::Tick, prelude::*, system::SystemChangeTick};
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

pub struct InterpolatePlugin;

#[derive(Default, Debug, Clone, Copy, Component)]
#[require(InterpolateState)]
pub struct Interpolate;

#[derive(Component, Clone, Copy, Debug, Default)]
pub enum InterpolateState {
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

pub fn start_interpolation(
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

pub fn end_interpolation(mut transforms: Query<(&mut Transform, &mut InterpolateState)>) {
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

impl Plugin for InterpolatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedFirst, end_interpolation);
        app.add_systems(
            RunFixedMainLoop,
            start_interpolation.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        );
    }
}
